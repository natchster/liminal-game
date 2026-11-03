//! The prop builders of `place_props`: each returns a group at the origin
//! and the surface height `top` for stacking. The second half
//! (core_props/tail.rs) calls some of them too. Re-exported by `core_props`.
use crate::base::records::Drawer;
use crate::core_props::{Built, PropsCtx};
use crate::prelude::*;
use crate::visuals::textures::{handwritten_tex, text_tex, HandOpts, TextTexOpts};
use std::collections::HashMap;

/// Reparent `object` under `parent`, keeping its world transform.
fn attach(parent: &Obj, object: &Obj) {
    parent.update_world_matrix(true, false);
    let mut m1 = parent.matrix_world().inverted();
    if let Some(op) = object.parent() {
        op.update_world_matrix(true, false);
        m1 = Mat4::mul(&m1, &op.matrix_world());
    }
    object.apply_matrix4(&m1);
    object.remove_from_parent();
    parent.add(object);
    object.update_world_matrix(false, true);
}

/// `handwritten_tex` options from the usual few fields
fn hand(color: &str, thickness: f64, jitter: f64, wobble: f64, w: f64, h: f64) -> HandOpts {
    HandOpts { color: color.into(), thickness, jitter, wobble, w, h, ..Default::default() }
}
/// `text_tex(text, ..)` with size, font, colour and glow
fn ttex(text: &str, w: f64, h: f64, font: &str, color: &str, glow: f64) -> Tex {
    text_tex(text, &TextTexOpts { w, h, font: font.into(), color: color.into(), glow, ..Default::default() })
}

/// Three drawers that slide out on use (`drawers`)
pub fn build_filing_cabinet(g: &mut Game, cx: &mut PropsCtx) -> Built {
    let _ = cx;
    let fg = group();
    let body = mesh(pg("Props__cabinet_body", box_geo!(0.6, 1.4, 0.55)), &g.mats.metal);
    body.position().y = 0.7;
    fg.add(&body);
    // top surface with slight lip
    let top = mesh(pg("Props__cabinet_top", box_geo!(0.62, 0.02, 0.57)), lambert!({ color: 0x484d54 }));
    top.position().y = 1.41;
    fg.add(&top);
    // feet
    // front pair forward into the body's toe-kick recess (y < 0.0325, z 0.25..0.275)
    // where they show; at z 0.22 all four were buried in the body
    for (sx, sz) in [(-0.25, -0.22), (0.25, -0.22), (-0.25, 0.245), (0.25, 0.245)] {
        let foot = mesh(pg("Props__cabinet_foot", box_geo!(0.06, 0.06, 0.06)), &g.mats.dark);
        foot.position().set(sx, 0.03, sz);
        fg.add(&foot);
    }
    // side seam lines
    for sx in [-0.3, 0.3] {
        let seam = mesh(box_geo!(0.005, 1.4, 0.01), &g.mats.dark);
        seam.position().set(sx, 0.7, 0.28);
        fg.add(&seam);
    }
    // brand plate on top
    let brand_plate = mesh(pg("Props__cabinet_brandPlate", box_geo!(0.14, 0.005, 0.05)), &g.mats.chrome);
    brand_plate.position().set(0.2, 1.42, 0.2);
    fg.add(&brand_plate);
    // rivets along side
    for r in 0..4 {
        let rivet = mesh(pg("Props__cabinet_rivet", sphere_geo!(0.008, 5, 4)), &g.mats.chrome);
        rivet.position().set(0.301, 0.2 + r as f64 * 0.32, 0);
        fg.add(&rivet);
    }
    // three drawers, each is a full open-topped box group that slides out on use
    let mut drawers = vec![];
    let drawer_inside_mat = lambert!({ color: 0x2a2d31 });
    let drawer_body_mat = lambert!({ color: 0x484d54 });
    for i in 0..3 {
        let drawer_group = group();
        // put group at drawer's front-flush position (front panel at local z=0.29)
        drawer_group.position().set(0, 0.28 + i as f64 * 0.45, 0);
        // FRONT PANEL (visible when closed)
        let front = mesh(pg("Props__cabinet_drawerFront", box_geo!(0.55, 0.34, 0.03)), lambert!({ color: 0x585d63, emissive: 0x0b0d10 }));
        front.position().set(0, 0, 0.29);
        drawer_group.add(&front);
        // recessed handle plate on front
        let handle_plate = mesh(pg("Props__cabinet_pullPlate", box_geo!(0.2, 0.08, 0.02)), lambert!({ color: 0x3a3f45 }));
        handle_plate.position().set(0, 0, 0.311);
        drawer_group.add(&handle_plate);
        let handle = mesh(pg("Props__cabinet_pull", box_geo!(0.14, 0.02, 0.05)), &g.mats.chrome);
        handle.position().set(0, 0, 0.34);
        drawer_group.add(&handle);
        let lock = mesh(pg("Props__cabinet_lock", cyl_geo!(0.014, 0.014, 0.02, 8)), &g.mats.chrome);
        lock.rotation().x = PI / 2.0;
        lock.position().set(0.17, 0, 0.315);
        drawer_group.add(&lock);
        let label = mesh(pg("Props__cabinet_label", box_geo!(0.16, 0.05, 0.01)), lambert!({ color: 0xe4dcc4 }));
        label.position().set(-0.14, 0.09, 0.311);
        drawer_group.add(&label);
        let label_text = mesh(box_geo!(0.11, 0.003, 0.008), &g.mats.dark);
        label_text.position().set(-0.14, 0.09, 0.318);
        drawer_group.add(&label_text);
        // BOX BODY: 4 sides + bottom, open on top; sized to fit inside the cabinet
        // interior. When closed, the box sits inside the cabinet body (invisible).
        // When drawer slides out, this whole box moves forward with the front.
        let box_depth = 0.48; // how deep into cabinet
        let box_width = 0.53;
        let box_height = 0.32;
        // bottom
        let bottom = mesh(pg("Props__cabinet_drawerBottom", box_geo!(box_width, 0.015, box_depth)), &drawer_body_mat);
        bottom.position().set(0, -box_height / 2.0 + 0.008, 0.29 - 0.015 - box_depth / 2.0);
        drawer_group.add(&bottom);
        // back wall
        let back = mesh(pg("Props__cabinet_drawerBack", box_geo!(box_width, box_height, 0.015)), &drawer_body_mat);
        back.position().set(0, 0, 0.29 - 0.015 - box_depth);
        drawer_group.add(&back);
        // left/right sides
        for s in [-1.0, 1.0] {
            let side = mesh(pg("Props__cabinet_drawerSide", box_geo!(0.015, box_height, box_depth)), &drawer_body_mat);
            side.position().set(s * box_width / 2.0, 0, 0.29 - 0.015 - box_depth / 2.0);
            drawer_group.add(&side);
        }
        // interior floor tint (looks worn/dark inside)
        let inner_floor = mesh(box_geo!(box_width - 0.03, 0.005, box_depth - 0.03), &drawer_inside_mat);
        inner_floor.position().set(0, -box_height / 2.0 + 0.016, 0.29 - 0.02 - box_depth / 2.0);
        drawer_group.add(&inner_floor);
        // hanging file rails inside (little metal lips)
        for s in [-1.0, 1.0] {
            let rail = mesh(pg("Props__cabinet_fileRail", box_geo!(0.008, 0.02, box_depth - 0.04)), &g.mats.chrome);
            rail.position().set(s * (box_width / 2.0 - 0.02), box_height / 2.0 - 0.02, 0.29 - 0.02 - box_depth / 2.0);
            drawer_group.add(&rail);
        }
        // stationary files sometimes stuffed in a drawer, visible only when open
        if rnd() < 0.6 {
            // the bound `irand(2, 4)` is deliberately redrawn every pass
            let mut f = 0i64;
            while f < irand(2, 4) {
                let file = mesh(
                    pg("Props__cabinet_folder", box_geo!(box_width - 0.06, box_height - 0.04, 0.02)),
                    lambert!({ color: [0xa89478u32, 0xc4a68a, 0x8a7860][irand(0, 2) as usize] }),
                );
                file.position().set(rand2(-0.02, 0.02), 0, 0.29 - 0.03 - 0.04 - f as f64 * 0.03);
                file.rotation().y = rand2(-0.05, 0.05);
                drawer_group.add(&file);
                f += 1;
            }
        }
        fg.add(&drawer_group);
        let content_roll = rnd();
        let content_type = if content_roll < 0.28 {
            "battery"
        } else if content_roll < 0.50 {
            "almond"
        } else if content_roll < 0.68 {
            "paper"
        } else {
            ""
        };
        drawers.push(Rc::new(RefCell::new(Drawer {
            mesh: drawer_group,
            opened: false,
            content_type: content_type.to_string(),
            content_taken: false,
            use_pos: Vec3::ZERO,
            f: HashMap::new(),
        })));
    }
    // occasional dent
    if rnd() < 0.4 {
        let dent = mesh(sphere_geo!(0.06, 6, 4), lambert!({ color: 0x2a2d31 }));
        dent.scale().set(1, 1, 0.2);
        let (x, y) = (rand2(-0.2, 0.2), rand2(0.3, 1.2));
        dent.position().set(x, y, 0.275);
        fg.add(&dent);
    }
    // occasional sticker
    if rnd() < 0.35 {
        let sticker = mesh(plane_geo!(0.1, 0.1), lambert!({ color: [0xd94040u32, 0x40a0d9, 0xd9c040][irand(0, 2) as usize] }));
        let (x, y) = (rand2(-0.2, 0.2), rand2(0.5, 1.3));
        sticker.position().set(x, y, 0.316);
        fg.add(&sticker);
    }
    Built { g: fg, top: 1.42, drawers }
}

pub fn build_cardboard_stack(g: &mut Game, cx: &mut PropsCtx) -> Built {
    let _ = cx;
    let cg = group();
    let stack = irand(2, 4);
    let labels = ["FRAGILE", "THIS SIDE UP", "BOOKS", "OFFICE 3", "DO NOT OPEN", "ATTIC", "HR", "MOM", "X-MAS", "TAX"];
    let mut y = 0.0;
    for _i in 0..stack {
        let sz = rand2(0.4, 0.6);
        let bx = mesh(pg("Props__cardboard_box", box_geo!(sz, 0.32, sz)), &g.mats.cardboard);
        let px = rand2(-0.08, 0.08);
        let pz = rand2(-0.08, 0.08);
        bx.position().set(px, y + 0.16, pz);
        bx.rotation().y = rand2(-0.3, 0.3);
        // cross-tape (top)
        let tape_top = mesh(box_geo!(sz + 0.01, 0.005, 0.05), lambert!({ color: 0xa08560 }));
        tape_top.position().y = 0.163;
        bx.add(&tape_top);
        let tape_side = mesh(box_geo!(0.05, 0.005, sz + 0.01), lambert!({ color: 0xa08560 }));
        tape_side.position().y = 0.163;
        bx.add(&tape_side);
        // dropped flap on top corner
        let flap = mesh(pg("Props__cardboard_flap", box_geo!(sz * 0.48, 0.005, sz * 0.48)), lambert!({ color: 0x8b6a42 }));
        flap.position().set(sz * 0.26, 0.16, -sz * 0.26);
        flap.rotation().z = rand2(-0.15, 0.15);
        bx.add(&flap);
        // hand-written label
        let lbl = mesh(
            plane_geo!(sz * 0.6, 0.09),
            basic_mat!({
                map: handwritten_tex(choice(&labels), &hand("rgba(20,10,5,0.95)", 4.0, 1.6, 1.2, 512.0, 96.0)),
                transparent: true,
            }),
        );
        lbl.position().set(0, 0.06, sz / 2.0 + 0.001);
        bx.add(&lbl);
        // sticker
        if rnd() < 0.5 {
            let sticker = mesh(circle_geo!(0.04, 8), lambert!({ color: [0xd94040u32, 0xd9c040, 0x40c96a][irand(0, 2) as usize] }));
            sticker.rotation().y = PI / 2.0;
            sticker.position().set(sz / 2.0 + 0.001, 0.08, rand2(-0.15, 0.15));
            bx.add(&sticker);
        }
        // corner damage — a triangle "torn" panel
        if rnd() < 0.4 {
            let dam = mesh(pg("Props__cardboard_tear", box_geo!(0.06, 0.06, 0.06)), lambert!({ color: 0x6a4a28 }));
            dam.position().set(sz / 2.0 - 0.02, 0.14, sz / 2.0 - 0.02);
            let (a, b, c) = (rand2(0.0, 1.0), rand2(0.0, 1.0), rand2(0.0, 1.0));
            dam.rotation().set(a, b, c);
            bx.add(&dam);
        }
        cg.add(&bx);
        y += 0.34;
    }
    Built::new(cg, y)
}

/// A-frame with printed warning face
pub fn build_wet_floor_sign(g: &mut Game, cx: &mut PropsCtx) -> Built {
    let _ = (g, cx);
    // the common 2-sided A-frame (Blender batch `r01`): two moulded yellow
    // panels hinged at the top, splayed 0.30 m at the foot, each printed
    // CAUTION / WET FLOOR in black, scuffed (docs/PROPS.md)
    use crate::assets::props::prop_part_as_modelled;
    const PANEL_H: f64 = 0.655;
    const FACE_Z: f64 = 0.0195; // the printed face, in the panel's frame
    let splay = (0.15f64 / PANEL_H).asin();
    let hinge_y = PANEL_H * splay.cos();
    let wg = group();
    let plastic = lambert!({ color: 0xc9a227 });
    let print_mat = lambert!({ map: &wet_sign_print_tex(), transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 });
    let panel_geo = prop_part_as_modelled("Real__wetSignPanel").unwrap_or_else(|| box_geo!(0.28, PANEL_H, 0.02));
    for side in [0.0, PI] {
        // each panel hangs from the hinge, its foot out to its own side
        let turn = group();
        turn.rotation().y = side;
        turn.position().y = hinge_y;
        let swing = group();
        swing.rotation().x = -splay;
        let panel = mesh(panel_geo.clone(), &plastic);
        panel.position().y = -PANEL_H;
        swing.add(&panel);
        let print = mesh(plane_geo!(0.222, 0.47), &print_mat);
        print.position().set(0, -PANEL_H + 0.2925, FACE_Z + 0.0009);
        swing.add(&print);
        turn.add(&swing);
        wg.add(&turn);
    }
    if let Some(h) = prop_part_as_modelled("Real__wetSignHinge") {
        let hinge = mesh(h, lambert!({ color: 0x9a7c1c }));
        hinge.position().y = hinge_y;
        wg.add(&hinge);
    }
    // the wet it stands in: a soft, uneven patch, darker than the floor
    let puddle = mesh(plane_geo!(0.9, 0.7), wet_patch_mat());
    puddle.rotation().x = -PI / 2.0;
    let px = rand2(-0.3, 0.3);
    let pz = rand2(-0.3, 0.3);
    puddle.position().set(px, 0.002, pz);
    wg.add(&puddle);
    Built::new(wg, 0.85)
}

/// The wet floor sign's print: CAUTION over the slipping figure, WET FLOOR
/// and the same in two more languages, black on the yellow it is laid on,
/// rubbed off at its edges and scratched (transparent where not printed)
fn wet_sign_print_tex() -> Tex {
    canvas_tex_opt(256, 544, false, |ctx, w, h| {
        ctx.clear_rect(0, 0, w, h);
        ctx.set_fill_style("#141210");
        ctx.set_text_align("center");
        ctx.set_text_baseline("alphabetic");
        ctx.set_font("bold 54px Helvetica Neue");
        ctx.fill_text_max("CAUTION", w / 2.0, 66, w - 16.0);
        // the slipping figure (ISO 7010 W011's person, in solid strokes)
        ctx.set_stroke_style("#141210");
        ctx.set_line_cap("round");
        ctx.set_line_join("round");
        ctx.set_line_width(19);
        let (cx0, cy0) = (w / 2.0 + 6.0, 255.0);
        ctx.begin_path();
        ctx.arc(cx0 + 34.0, cy0 - 104.0, 19, 0, 7);
        ctx.fill();
        ctx.begin_path(); // the body, falling back
        ctx.move_to(cx0 + 22.0, cy0 - 74.0);
        ctx.line_to(cx0 - 8.0, cy0 + 4.0);
        ctx.stroke();
        ctx.begin_path(); // the leg thrown up
        ctx.move_to(cx0 - 8.0, cy0 + 4.0);
        ctx.line_to(cx0 + 40.0, cy0 + 22.0);
        ctx.line_to(cx0 + 72.0, cy0 - 4.0);
        ctx.stroke();
        ctx.begin_path(); // the leg under it
        ctx.move_to(cx0 - 8.0, cy0 + 4.0);
        ctx.line_to(cx0 - 26.0, cy0 + 50.0);
        ctx.line_to(cx0 - 6.0, cy0 + 72.0);
        ctx.stroke();
        ctx.begin_path(); // the arms thrown out
        ctx.move_to(cx0 - 64.0, cy0 - 70.0);
        ctx.line_to(cx0 + 12.0, cy0 - 50.0);
        ctx.line_to(cx0 + 78.0, cy0 - 92.0);
        ctx.stroke();
        // the floor, and the wet on it
        ctx.set_line_width(7);
        ctx.begin_path();
        ctx.move_to(24, cy0 + 86.0);
        ctx.line_to(w - 24.0, cy0 + 86.0);
        ctx.stroke();
        ctx.set_line_width(4);
        for k in 0..3 {
            let x = 52.0 + k as f64 * 58.0;
            ctx.begin_path();
            ctx.move_to(x, cy0 + 100.0);
            ctx.bezier_curve_to(x + 10.0, cy0 + 94.0, x + 20.0, cy0 + 106.0, x + 30.0, cy0 + 100.0);
            ctx.stroke();
        }
        ctx.set_font("bold 40px Helvetica Neue");
        ctx.fill_text_max("WET FLOOR", w / 2.0, 418, w - 20.0);
        ctx.set_font("bold 25px Helvetica Neue");
        ctx.fill_text_max("PISO MOJADO", w / 2.0, 458, w - 20.0);
        ctx.set_font("bold 19px Helvetica Neue");
        ctx.fill_text_max("ATTENTION SOL GLISSANT", w / 2.0, 490, w - 20.0);
        // worn: rubbed thin where hands and mops touch it, and scratched
        ctx.set_global_composite_operation("destination-out");
        for _ in 0..90 {
            let (x, y) = (rnd() * w, rnd() * h);
            ctx.set_global_alpha(0.15 + 0.35 * rnd());
            ctx.begin_path();
            ctx.ellipse(x, y, 2.0 + rnd() * 9.0, 1.0 + rnd() * 4.0, rnd() * 3.0, 0, 7);
            ctx.fill();
        }
        ctx.set_line_width(1.2);
        for _ in 0..40 {
            let (x, y) = (rnd() * w, rnd() * h);
            let a = rnd() * 6.3;
            let l = 8.0 + rnd() * 40.0;
            ctx.set_global_alpha(0.4 + 0.5 * rnd());
            ctx.begin_path();
            ctx.move_to(x, y);
            ctx.line_to(x + a.cos() * l, y + a.sin() * l);
            ctx.stroke();
        }
        // the foot, where the mop and the shoes rub it
        let grad = ctx.create_linear_gradient(0, h - 70.0, 0, h);
        grad.add_color_stop(0, "rgba(0,0,0,0)");
        grad.add_color_stop(1, "rgba(0,0,0,0.6)");
        ctx.set_global_alpha(1);
        ctx.set_fill_style(grad);
        ctx.fill_rect(0, h - 70.0, w, 70);
        ctx.set_global_composite_operation("source-over");
    })
}

/// A wet patch on the floor: dark, uneven, soft at its edge (the floor
/// shows through it)
fn wet_patch_mat() -> Mat {
    let tex = canvas_tex_opt(128, 128, false, |ctx, w, h| {
        ctx.clear_rect(0, 0, w, h);
        for _ in 0..14 {
            let (x, y) = (w * (0.25 + 0.5 * rnd()), h * (0.25 + 0.5 * rnd()));
            let r = w * (0.12 + 0.16 * rnd());
            let gr = ctx.create_radial_gradient(x, y, 0, x, y, r);
            gr.add_color_stop(0, "rgba(20,18,14,0.5)");
            gr.add_color_stop(0.6, "rgba(20,18,14,0.35)");
            gr.add_color_stop(1, "rgba(20,18,14,0)");
            ctx.set_fill_style(gr);
            ctx.begin_path();
            ctx.ellipse(x, y, r, r * (0.6 + 0.4 * rnd()), rnd() * 3.0, 0, 7);
            ctx.fill();
        }
    });
    lambert!({ map: &tex, transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 })
}

/// A bank of 2–3 lockers, sometimes one ajar
pub fn build_locker(g: &mut Game, cx: &mut PropsCtx) -> Built {
    let _ = cx;
    let lg = group();
    let doors = irand(2, 3);
    let df = doors as f64;
    // shared back panel
    let back = mesh(pg("Props__locker_back", box_geo!(df * 0.44 + 0.04, 1.72, 0.05)), lambert!({ color: 0x2a2e35 }));
    back.position().set(0, 0.86, -0.24);
    lg.add(&back);
    // shared top
    let top_shelf = mesh(pg("Props__locker_top", box_geo!(df * 0.44 + 0.04, 0.04, 0.55)), &g.mats.metal);
    top_shelf.position().set(0, 1.73, 0);
    lg.add(&top_shelf);
    // shared feet
    for i in 0..doors + 1 {
        let foot = mesh(pg("Props__locker_rail", box_geo!(0.06, 0.06, 0.5)), lambert!({ color: 0x1a1c20 }));
        foot.position().set((i as f64 - df / 2.0) * 0.44, 0.03, 0);
        lg.add(&foot);
    }
    for i in 0..doors {
        let pos_x = (i as f64 - (df - 1.0) / 2.0) * 0.44;
        let face_start = lg.child_count(); // this unit's door + face parts (ajar leaf below)
        let door = mesh(pg("Props__locker_unit", box_geo!(0.42, 1.7, 0.5)), &g.mats.metal);
        door.position().set(pos_x, 0.85, 0);
        lg.add(&door);
        // vent slats
        for j in 0..4 {
            let slat = mesh(pg("Props__locker_louvre", box_geo!(0.28, 0.02, 0.03)), lambert!({ color: 0x101215 }));
            slat.position().set(pos_x, 1.55 - j as f64 * 0.06, 0.252);
            lg.add(&slat);
        }
        // recessed handle plate
        let hplate = mesh(pg("Props__locker_handlePlate", box_geo!(0.14, 0.16, 0.015)), lambert!({ color: 0x2a2e35 }));
        hplate.position().set(pos_x, 0.85, 0.253);
        lg.add(&hplate);
        // handle (vertical bar)
        let handle = mesh(pg("Props__locker_handle", box_geo!(0.03, 0.12, 0.025)), &g.mats.chrome);
        handle.position().set(pos_x + 0.03, 0.85, 0.26);
        lg.add(&handle);
        // padlock loop
        let lock_body = mesh(pg("Props__locker_padlock", box_geo!(0.05, 0.06, 0.03)), lambert!({ color: 0xd0c060 }));
        lock_body.position().set(pos_x - 0.03, 0.79, 0.27);
        lg.add(&lock_body);
        let lock_loop = mesh(pg("Props__locker_shackle", torus_geo!(0.02, 0.005, 4, 8)), &g.mats.chrome);
        // upright shackle, its lower half in the lock body (top 0.82); it lay flat 1.5 cm above it
        lock_loop.position().set(pos_x - 0.03, 0.83, 0.27);
        lg.add(&lock_loop);
        // paper name tag
        let tag = mesh(
            plane_geo!(0.16, 0.06),
            basic_mat!({
                map: handwritten_tex(choice(&["ELIAS", "M. R.", "7-B", "DO NOT", "M.KANE", "EMPTY", "STAFF", "YOU"]),
                    &hand("rgba(20,15,10,0.9)", 3.0, 1.5, 1.2, 256.0, 128.0)),
                transparent: true,
            }),
        );
        tag.position().set(pos_x, 1.28, 0.253);
        lg.add(&tag);
        // occasionally ajar: a door leaf swung about its hinge (left front edge,
        // where the part's knuckles are). The column is flattened to a 4 cm leaf
        // and its louvres / handle / lock / tag ride on it; spinning the whole
        // 0.5 m column about its centre left them floating in the air.
        let mut leaf: Option<Obj> = None;
        if rnd() < 0.35 {
            door.scale().z = 0.08;
            door.position().z = 0.23;
            let lf = group();
            lf.position().set(pos_x - 0.21, 0, 0.25);
            lg.add(&lf);
            let kids: Vec<Obj> = lg.children()[face_start..].to_vec();
            for c in &kids {
                if !c.ptr_eq(&lf) {
                    attach(&lf, c);
                }
            }
            lf.rotation().y = -0.6;
            // sometimes with a coat hanging inside
            if rnd() < 0.5 {
                let coat = mesh(pg("Props__locker_coat", box_geo!(0.24, 0.7, 0.08)), lambert!({ color: 0x2a2020 }));
                coat.position().set(pos_x, 1.15, -0.05);
                lg.add(&coat);
            }
            leaf = Some(lf);
        }
        // rust patches
        if rnd() < 0.4 {
            let patch = mesh(circle_geo!(rand2(0.03, 0.07), 8), &g.mats.rust);
            let px = pos_x + rand2(-0.15, 0.15);
            let py = rand2(0.2, 1.5);
            patch.position().set(px, py, 0.251);
            if let Some(lf) = &leaf {
                patch.position().x -= pos_x - 0.21;
                patch.position().z -= 0.25;
                lf.add(&patch);
            } else {
                lg.add(&patch);
            }
        }
    }
    Built::new(lg, 1.74)
}

/// Wall-mounted
pub fn build_fire_extinguisher(g: &mut Game, cx: &mut PropsCtx) -> Built {
    let _ = cx;
    let eg = group();
    // wall mounting bracket
    let bracket = mesh(pg("Props__extinguisher_bracket", box_geo!(0.24, 0.06, 0.04)), &g.mats.metal);
    bracket.position().set(0, 0.8, -0.11);
    eg.add(&bracket);
    // strap holding tank
    let strap = mesh(pg("Props__extinguisher_strap", box_geo!(0.05, 0.24, 0.24)), lambert!({ color: 0x1a1a1a }));
    strap.rotation().z = PI / 2.0; // ring axis x -> y: a band round the tank, not a slab through it
    strap.position().set(0, 0.75, 0);
    eg.add(&strap);
    // body
    let body = mesh(pg("Props__extinguisher_tank", cyl_geo!(0.11, 0.11, 0.5, 14)), &g.mats.plastic_red);
    body.position().y = 0.7;
    eg.add(&body);
    // white/red label wraps around body
    let label_tex = canvas_tex_opt(256, 128, true, |ctx, w, h| {
        ctx.set_fill_style("#e8e2d4");
        ctx.fill_rect(0, 0, w, h);
        ctx.set_fill_style("#a02020");
        ctx.fill_rect(0, 0, w, 24);
        ctx.fill_rect(0, h - 24.0, w, 24);
        ctx.set_fill_style("#101010");
        ctx.set_text_align("center");
        ctx.set_font("bold 20px Impact, sans-serif");
        ctx.fill_text("ABC DRY CHEMICAL", 128, 66);
        ctx.set_font("bold 12px Impact, sans-serif");
        ctx.fill_text("5 LB CAPACITY", 128, 88);
    });
    label_tex.t().repeat.set(1, 1);
    let label = mesh(cyl_geo!(0.112, 0.112, 0.22, 14, 1, true), lambert!({ map: &label_tex, side: DoubleSide }));
    label.position().y = 0.68;
    eg.add(&label);
    // shoulder
    let shoulder = mesh(pg("Props__extinguisher_shoulder", cyl_geo!(0.08, 0.11, 0.07, 12)), &g.mats.plastic_red);
    shoulder.position().y = 0.98;
    eg.add(&shoulder);
    // neck + valve
    let neck = mesh(pg("Props__extinguisher_valve", cyl_geo!(0.045, 0.06, 0.05, 10)), &g.mats.chrome);
    neck.position().y = 1.03;
    eg.add(&neck);
    // handle (two levers)
    let handle = mesh(pg("Props__extinguisher_carryHandle", box_geo!(0.09, 0.02, 0.16)), lambert!({ color: 0x2a2a2a }));
    handle.position().set(0, 1.09, 0);
    eg.add(&handle);
    let handle2 = mesh(pg("Props__extinguisher_lever", box_geo!(0.09, 0.02, 0.16)), lambert!({ color: 0x2a2a2a }));
    handle2.position().set(0, 1.13, 0);
    eg.add(&handle2);
    // pin with ring
    let pin = mesh(pg("Props__extinguisher_pin", cyl_geo!(0.006, 0.006, 0.06, 6)), &g.mats.chrome);
    pin.rotation().x = PI / 2.0;
    pin.position().set(-0.05, 1.11, 0.05);
    eg.add(&pin);
    let ring = mesh(pg("Props__extinguisher_pullRing", torus_geo!(0.015, 0.004, 4, 8)), &g.mats.chrome);
    ring.position().set(-0.05, 1.11, 0.09);
    ring.rotation().y = PI / 2.0;
    eg.add(&ring);
    // pressure gauge
    let gauge = mesh(pg("Props__extinguisher_gauge", cyl_geo!(0.028, 0.028, 0.02, 10)), &g.mats.chrome);
    gauge.rotation().x = PI / 2.0;
    gauge.position().set(0.06, 1.03, 0.06);
    eg.add(&gauge);
    let gauge_face = mesh(circle_geo!(0.025, 10), basic_mat!({ color: 0xe0d8c0 }));
    // facing out (+z) in the gauge bezel's recess (face z 0.066, lip 0.07); it faced the floor
    gauge_face.position().set(0.06, 1.03, 0.0665);
    eg.add(&gauge_face);
    // hose
    let hose = mesh(pg("Props__extinguisher_hose", cyl_geo!(0.012, 0.012, 0.3, 6)), lambert!({ color: 0x1a1a1a }));
    hose.rotation().z = 0.6;
    hose.position().set(0.11, 0.94, 0.08);
    eg.add(&hose);
    // nozzle
    let nozzle = mesh(pg("Props__extinguisher_nozzle", box_geo!(0.05, 0.05, 0.14)), &g.mats.dark);
    nozzle.position().set(0.21, 0.82, 0.14);
    nozzle.rotation().x = 0.6;
    eg.add(&nozzle);
    Built::new(eg, 1.15)
}

/// Overflowing, with butts and a stain
pub fn build_trash_can(g: &mut Game, cx: &mut PropsCtx) -> Built {
    let _ = cx;
    let tg = group();
    // body with darker inner
    let body = mesh(pg("Props__trashCan_body", cyl_geo!(0.22, 0.19, 0.65, 14)), &g.mats.metal);
    body.position().y = 0.325;
    tg.add(&body);
    let inner = mesh(cyl_geo!(0.2, 0.17, 0.63, 14, 1, true), lambert!({ color: 0x1a1c20, side: BackSide }));
    inner.position().y = 0.335;
    tg.add(&inner);
    // horizontal ribs on outside
    for r in 0..3 {
        let rf = r as f64;
        let rib = mesh(pg("Props__trashCan_rib", torus_geo!(0.215, 0.008, 4, 14)), lambert!({ color: 0x2a2f34 }));
        rib.rotation().x = PI / 2.0;
        // sized to the tapered body at its height (r 0.19 at the foot .. 0.22 at the rim)
        let rib_k = (0.19 + 0.03 * (0.15 + rf * 0.2) / 0.65 + 0.006) / 0.215;
        rib.scale().set(rib_k, rib_k, 1);
        rib.position().y = 0.15 + rf * 0.2;
        tg.add(&rib);
    }
    // rim
    let rim = mesh(pg("Props__trashCan_rim", torus_geo!(0.22, 0.012, 6, 16)), &g.mats.chrome);
    rim.rotation().x = PI / 2.0;
    rim.position().y = 0.65;
    tg.add(&rim);
    // overflow: crumpled paper + a banana peel-ish thing + a can
    for _i in 0..4 {
        let paper = mesh(pg("Props__trash_paperBall", sphere_geo!(rand2(0.05, 0.09), 6, 4)), &g.mats.paper);
        let x = rand2(-0.15, 0.15);
        let y = 0.68 + rand2(0.0, 0.1);
        let z = rand2(-0.15, 0.15);
        paper.position().set(x, y, z);
        paper.scale().y = 0.6;
        tg.add(&paper);
    }
    // crushed can
    let can = mesh(pg("Props__trash_crushedCan", cyl_geo!(0.03, 0.03, 0.09, 8)), lambert!({ color: 0x2a5aa0 }));
    let x = rand2(-0.1, 0.1);
    let z = rand2(-0.1, 0.1);
    can.position().set(x, 0.72, z);
    can.rotation().z = rand2(-0.6, 0.6);
    tg.add(&can);
    // banana peel — three curled strips
    if rnd() < 0.5 {
        let peel_mat = lambert!({ color: 0xa89a3a });
        for p in 0..3 {
            let strip = mesh(pg("Props__trash_peel", box_geo!(0.02, 0.09, 0.02)), &peel_mat);
            let a = (p as f64 / 3.0) * PI * 2.0;
            strip.position().set(a.cos() * 0.03, 0.72, a.sin() * 0.03 + rand2(-0.1, 0.1));
            let rx = rand2(-0.4, 0.4);
            let rz = rand2(-0.4, 0.4);
            strip.rotation().set(rx, a, rz);
            tg.add(&strip);
        }
    }
    // takeout container lid poking out
    if rnd() < 0.4 {
        let lid = mesh(pg("Props__trash_lid", box_geo!(0.12, 0.02, 0.08)), lambert!({ color: 0xf0e8d0 }));
        let x = rand2(-0.08, 0.08);
        let z = rand2(-0.08, 0.08);
        lid.position().set(x, 0.75, z);
        let (rx, ry, rz) = (rand2(-0.4, 0.4), rand2(0.0, 7.0), rand2(-0.4, 0.4));
        lid.rotation().set(rx, ry, rz);
        tg.add(&lid);
    }
    // cigarette butts scattered outside
    for _i in 0..3 {
        let butt = mesh(pg("Props__trash_butt", cyl_geo!(0.008, 0.008, 0.03, 6)), lambert!({ color: 0xd8d0b8 }));
        butt.rotation().z = PI / 2.0 + rand2(-0.5, 0.5);
        let x = rand2(-0.35, 0.35);
        let z = rand2(-0.35, 0.35);
        butt.position().set(x, 0.011, z);
        tg.add(&butt);
        // burn tip
        let tip = mesh(pg("Props__trash_buttTip", cyl_geo!(0.008, 0.008, 0.008, 6)), lambert!({ color: 0x1a1008 }));
        let br = butt.rot();
        tip.rotation().copy(&br);
        tip.position().copy(&butt.pos());
        tip.position().x += (br.z + PI / 2.0).sin() * 0.017;
        tg.add(&tip);
    }
    // dark liquid stain around base
    let stain = mesh(circle_geo!(0.35, 12), basic_mat!({ color: 0x1a1006, transparent: true, opacity: 0.55 }));
    stain.rotation().x = -PI / 2.0;
    let x = rand2(-0.1, 0.1);
    let z = rand2(-0.1, 0.1);
    stain.position().set(x, 0.003, z);
    tg.add(&stain);
    // scattered debris on floor
    for _i in 0..3 {
        let debris = mesh(
            pg("Props__trash_debris", box_geo!(rand2(0.03, 0.08), 0.02, rand2(0.03, 0.08))),
            if rnd() < 0.5 { &g.mats.paper } else { &g.mats.cardboard },
        );
        let x = rand2(-0.32, 0.32);
        let z = rand2(-0.32, 0.32);
        debris.position().set(x, 0.01, z);
        debris.rotation().y = rand2(0.0, 7.0);
        tg.add(&debris);
    }
    Built::new(tg, 0.82)
}

pub fn build_rubber_duck(g: &mut Game, cx: &mut PropsCtx) -> Built {
    let _ = cx;
    let dg = group();
    let body = mesh(pg("Props__duck_body", sphere_geo!(0.09, 12, 8)), &g.mats.plastic_yellow);
    body.scale().set(1.2, 0.9, 1.4);
    body.position().y = 0.09;
    dg.add(&body);
    let head = mesh(pg("Props__duck_head", sphere_geo!(0.06, 10, 8)), &g.mats.plastic_yellow);
    head.position().set(0, 0.19, 0.09);
    dg.add(&head);
    let beak = mesh(pg("Props__duck_beak", cone_geo!(0.025, 0.055, 6)), &g.mats.plastic_orange);
    beak.rotation().x = PI / 2.0;
    beak.position().set(0, 0.19, 0.14);
    dg.add(&beak);
    // eyes
    for s in [-1.0, 1.0] {
        let eye = mesh(pg("Props__duck_eye", sphere_geo!(0.012, 6, 4)), &g.mats.dark);
        eye.position().set(s * 0.025, 0.22, 0.12);
        dg.add(&eye);
    }
    Built::new(dg, 0.26)
}

pub fn build_traffic_cone(g: &mut Game, cx: &mut PropsCtx) -> Built {
    let _ = cx;
    let cg = group();
    let base = mesh(pg("Props__cone_base", box_geo!(0.34, 0.03, 0.34)), &g.mats.plastic_orange);
    base.position().y = 0.015;
    cg.add(&base);
    let cone = mesh(pg("Props__cone_body", cone_geo!(0.16, 0.55, 10)), &g.mats.plastic_orange);
    cone.position().y = 0.30;
    cg.add(&cone);
    // white reflective band
    let band = mesh(pg("Props__cone_band", cyl_geo!(0.13, 0.15, 0.05, 10)), &g.mats.plastic_white);
    band.scale().set(0.71, 1, 0.71); // hugs the cone body (r 0.089..0.101 there), not a 5 cm collar
    band.position().y = 0.28;
    cg.add(&band);
    Built::new(cg, 0.58)
}

pub fn build_wheelchair(g: &mut Game, cx: &mut PropsCtx) -> Built {
    let _ = cx;
    let wg = group();
    // seat cushion — separate mesh for the cushion effect
    let seat_frame = mesh(pg("Props__wheelchair_seatFrame", box_geo!(0.5, 0.04, 0.48)), &g.mats.chrome);
    seat_frame.position().y = 0.48;
    wg.add(&seat_frame);
    let cushion = mesh(pg("Props__wheelchair_cushion", box_geo!(0.42, 0.08, 0.42)), &g.mats.plastic_red);
    cushion.position().y = 0.53;
    wg.add(&cushion);
    // seat back
    let back = mesh(pg("Props__wheelchair_back", box_geo!(0.42, 0.55, 0.05)), &g.mats.plastic_red);
    back.position().set(0, 0.82, -0.22);
    wg.add(&back);
    let back_frame = mesh(pg("Props__wheelchair_backFrame", box_geo!(0.5, 0.62, 0.03)), &g.mats.chrome);
    back_frame.position().set(0, 0.85, -0.24);
    wg.add(&back_frame);
    // arm rests
    for s in [-1.0, 1.0] {
        let arm_pad = mesh(pg("Props__wheelchair_armPad", box_geo!(0.06, 0.04, 0.32)), lambert!({ color: 0x2a1a10 }));
        arm_pad.position().set(s * 0.25, 0.7, -0.03);
        wg.add(&arm_pad);
        let arm_frame = mesh(pg("Props__wheelchair_armPost", box_geo!(0.02, 0.22, 0.02)), &g.mats.chrome);
        arm_frame.position().set(s * 0.25, 0.6, 0.11);
        wg.add(&arm_frame);
    }
    // push handles at back
    for s in [-1.0, 1.0] {
        let grip = mesh(pg("Props__wheelchair_grip", cyl_geo!(0.02, 0.02, 0.08, 8)), lambert!({ color: 0x2a1a10 }));
        grip.rotation().z = PI / 2.0;
        grip.position().set(s * 0.22, 1.18, -0.24);
        wg.add(&grip);
        let grip_up = mesh(pg("Props__wheelchair_handlePost", cyl_geo!(0.018, 0.018, 0.22, 8)), &g.mats.chrome);
        grip_up.position().set(s * 0.22, 1.08, -0.24);
        wg.add(&grip_up);
    }
    // large rear wheels with spokes
    for s in [-1.0, 1.0] {
        let rim = mesh(pg("Props__wheelchair_tire", torus_geo!(0.28, 0.03, 6, 24)), &g.mats.dark);
        // axle at the tyre's outer radius (0.31) so it stands on the floor with the casters
        rim.position().set(s * 0.28, 0.31, 0);
        rim.rotation().y = PI / 2.0;
        wg.add(&rim);
        // outer push rim: smaller than the tyre (it was R 0.31, past the tyre and 4 cm under the floor)
        let push = mesh(pg("Props__wheelchair_pushRim", torus_geo!(0.31, 0.012, 4, 20)), &g.mats.chrome);
        push.scale().set(0.87, 0.87, 1);
        push.position().set(s * 0.30, 0.31, 0);
        push.rotation().y = PI / 2.0;
        wg.add(&push);
        // spokes
        for sp in 0..8 {
            let spoke = mesh(pg("Props__wheelchair_spoke", box_geo!(0.005, 0.55, 0.005)), &g.mats.chrome);
            spoke.position().set(s * 0.28, 0.31, 0);
            spoke.rotation().x = (sp as f64 / 8.0) * PI;
            wg.add(&spoke);
        }
        // hub
        let hub = mesh(pg("Props__wheelchair_hub", cyl_geo!(0.04, 0.04, 0.06, 8)), &g.mats.chrome);
        hub.rotation().z = PI / 2.0;
        hub.position().set(s * 0.28, 0.31, 0);
        wg.add(&hub);
        // small caster wheel (outer radius 0.10: centre 0.10 so it meets the floor)
        let small = mesh(pg("Props__wheelchair_caster", torus_geo!(0.08, 0.02, 6, 12)), &g.mats.dark);
        small.position().set(s * 0.22, 0.10, 0.28);
        small.rotation().y = PI / 2.0;
        wg.add(&small);
        let fork_t = mesh(pg("Props__wheelchair_fork", box_geo!(0.02, 0.12, 0.02)), &g.mats.chrome);
        fork_t.position().set(s * 0.22, 0.16, 0.28);
        wg.add(&fork_t);
        // footrest
        let foot = mesh(pg("Props__wheelchair_footrest", box_geo!(0.14, 0.03, 0.09)), &g.mats.chrome);
        foot.position().set(s * 0.12, 0.15, 0.32);
        wg.add(&foot);
    }
    // frame side bars
    for s in [-1.0, 1.0] {
        let bar = mesh(pg("Props__wheelchair_sideBar", box_geo!(0.03, 0.7, 0.03)), &g.mats.chrome);
        bar.position().set(s * 0.24, 0.35, 0.12);
        wg.add(&bar);
    }
    // cross brace under seat
    let brace = mesh(pg("Props__wheelchair_crossBrace", box_geo!(0.48, 0.03, 0.03)), &g.mats.chrome);
    brace.position().set(0, 0.25, 0.05);
    wg.add(&brace);
    Built::new(wg, 1.2)
}

/// Lying on its back
pub fn build_baby_doll(g: &mut Game, cx: &mut PropsCtx) -> Built {
    let _ = cx;
    let dg = group();
    let skin_mat = lambert!({ color: 0xd8b8a0 });
    let body = mesh(pg("Props__doll_body", capsule_geo!(0.09, 0.2, 4, 8)), &skin_mat);
    body.rotation().z = PI / 2.0;
    body.position().y = 0.09;
    dg.add(&body);
    let head = mesh(pg("Props__doll_head", sphere_geo!(0.09, 12, 8)), &skin_mat);
    // lying on its back: the head part's face (+x) turned to look up (+y), its
    // forehead (+y) toward the crown (+x); it stared along the body axis
    head.rotation().set(PI, 0, -PI / 2.0);
    head.position().set(0.22, 0.09, 0);
    dg.add(&head);
    // creepy dark eye sockets, in the turned head's sockets
    for s in [-1.0, 1.0] {
        let eye = mesh(pg("Props__doll_eye", sphere_geo!(0.02, 6, 4)), &g.mats.dark);
        eye.position().set(0.24, 0.17, s * 0.03);
        dg.add(&eye);
    }
    for s in [-1.0, 1.0] {
        let arm = mesh(pg("Props__doll_arm", capsule_geo!(0.028, 0.16, 4, 6)), &skin_mat);
        // shoulder end on the floor (at y 0.05 it reached 4.8 cm under it)
        arm.position().set(-0.05, 0.10, s * 0.11);
        arm.rotation().z = 0.5;
        dg.add(&arm);
        let leg = mesh(pg("Props__doll_leg", capsule_geo!(0.032, 0.14, 4, 6)), &skin_mat);
        leg.position().set(-0.18, 0.05, s * 0.06);
        leg.rotation().z = 1.4;
        dg.add(&leg);
    }
    Built::new(dg, 0.22)
}

/// Lid open, clothes spilling
pub fn build_suitcase(g: &mut Game, cx: &mut PropsCtx) -> Built {
    let _ = (g, cx);
    let sg = group();
    // lid open
    let bottom = mesh(pg("Props__suitcase_base", box_geo!(0.55, 0.14, 0.4)), lambert!({ color: 0x5a3d22 }));
    bottom.position().y = 0.07;
    sg.add(&bottom);
    let lid = mesh(pg("Props__suitcase_lid", box_geo!(0.55, 0.14, 0.4)), lambert!({ color: 0x4a3319 }));
    lid.position().set(0, 0.14, -0.19);
    lid.rotation().x = -0.9;
    sg.add(&lid);
    // spilled contents (clothes as small shapes)
    for _i in 0..5 {
        let cloth_mat = lambert!({ color: [0x7a3a2au32, 0x2a3a5a, 0xa08560, 0x555555][irand(0, 3) as usize] });
        let cloth = mesh(pg("Props__suitcase_clothes", sphere_geo!(rand2(0.04, 0.08), 6, 4)), &cloth_mat);
        let x = rand2(-0.2, 0.2);
        let y = 0.15 + rand2(0.0, 0.05);
        let z = rand2(-0.1, 0.1);
        cloth.position().set(x, y, z);
        sg.add(&cloth);
    }
    Built::new(sg, 0.4)
}

/// A broken washroom mirror on the wall (Blender batch `r05`; `yaw` is unused)
pub fn build_broken_mirror(g: &mut Game, cx: &mut PropsCtx, yaw: f64) -> Built {
    let _ = (g, cx, yaw);
    // A 24 x 36 in stainless channel-frame mirror (the Bobrick B-165 class)
    // on its hanger, flat on the wall: silvered glass that gives back the
    // room's light (`Patch::Glass`), its silver going black from the edges
    // and in pinholes, dust and dried splashes on it, broken where something
    // hit it (radial cracks from the blow, rings between them, now and then
    // a shard gone), every crack inside the glass (docs/PROPS.md).
    use crate::assets::props::prop_part_as_modelled;
    // the placer stands the group 0.11 m out from the wall's face (the wall
    // spot's 6 cm and its own 5)
    const WALL: f64 = -0.11;
    // the old cracks' draws: four, a start and five steps each; they break the glass
    let mut d = [[0.0f64; 12]; 4];
    for c in d.iter_mut() {
        c[0] = rand2(-0.3, 0.3);
        c[1] = rand2(-0.5, 0.5);
        for s in 0..5 {
            c[2 + 2 * s] = rand2(-0.15, 0.15);
            c[3 + 2 * s] = rand2(-0.2, 0.2);
        }
    }
    let brk = MirrorBreak::from_draws(&d);
    let mg = group();
    // the frame: bright-polished stainless gone dull
    if let Some(geo) = prop_part_as_modelled("Real__mirrorFrame") {
        let frame = mesh(geo, phong!({ color: 0x55585a, specular: 0x8a8d90, shininess: 48 }));
        frame.position().z = WALL;
        mg.add(&frame);
    }
    // the black paint on the glass's back, seen where the silver has gone
    let back = mesh(plane_geo!(MIRROR_GLASS[0] + 0.01, MIRROR_GLASS[1] + 0.01), lambert!({ color: 0x141312 }));
    back.position().z = WALL + 0.0112;
    mg.add(&back);
    // the glass, its edges under the frame's lip
    let glass = mesh(plane_geo!(MIRROR_GLASS[0] + MIRROR_UNDER, MIRROR_GLASS[1] + MIRROR_UNDER), mirror_glass_mat(&brk));
    glass.position().z = WALL + MIRROR_GLASS[2];
    glass.set_render_order(2);
    mg.add(&glass);
    // what is on its face: dust, dried splashes, the cracks catching the light
    let film_mat = lambert!({
        map: &mirror_film_tex(&brk), transparent: true, depthWrite: false,
        polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2,
    });
    let film = mesh(plane_geo!(MIRROR_GLASS[0], MIRROR_GLASS[1]), &film_mat);
    film.position().z = WALL + MIRROR_GLASS[2] + 0.0003;
    film.set_render_order(3);
    mg.add(&film);
    Built::new(mg, 1.4)
}

/// The mirror's glass as seen inside its frame (width, height) and its
/// face's distance from the wall (`Real__mirrorFrame`'s hem, r05.py)
const MIRROR_GLASS: [f64; 3] = [0.584, 0.888, 0.0172];
/// How much more of the glass runs under the frame's lip (both sides)
const MIRROR_UNDER: f64 = 0.006;

/// A mirror's breakage, in metres on its glass (x right and y up from its
/// middle), worked out from the old cracks' draws and a generator seeded
/// by them (no draws of its own), so the silver and the film agree
#[derive(Clone, Default)]
struct MirrorBreak {
    /// each crack: its points, and how heavy its line is (1 a fine one)
    lines: Vec<(Vec<[f64; 2]>, f64)>,
    /// where it was struck, and how wide the crushed spot is
    hits: Vec<([f64; 2], f64)>,
    /// a shard fallen out by the blow (empty if none)
    hole: Vec<[f64; 2]>,
}

impl MirrorBreak {
    fn from_draws(d: &[[f64; 12]; 4]) -> MirrorBreak {
        let (hw, hh) = (MIRROR_GLASS[0] / 2.0, MIRROR_GLASS[1] / 2.0);
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for v in d.iter().flatten() {
            h ^= v.to_bits();
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
        let mut r = SplitMix64::new(0, h);
        let mut b = MirrorBreak::default();
        let inside = |p: [f64; 2]| p[0].abs() < hw + 0.004 && p[1].abs() < hh + 0.004;
        // a crack run from `p` toward `a`, wandering a little, to the glass's
        // edge or until it stops (`reach`)
        let run = |r: &mut SplitMix64, p: [f64; 2], a: f64, reach: f64| -> Vec<[f64; 2]> {
            let mut pts = vec![p];
            let (mut q, mut a, mut l) = (p, a, 0.0);
            while l < reach && inside(q) {
                let step = r.range(0.012, 0.03);
                a += r.range(-0.11, 0.11);
                q = [q[0] + a.cos() * step, q[1] + a.sin() * step];
                l += step;
                pts.push(q);
            }
            pts
        };
        let kind = (d[1][0] + 0.3) / 0.6;
        if kind < 0.74 {
            // struck: the blow's centre, chest high or a little above
            let p = [d[0][0] * 0.62, d[0][1] * 0.5 + 0.05];
            let crush = r.range(0.007, 0.014);
            b.hits.push((p, crush));
            let n = 6 + ((d[1][1] + 0.5) * 5.0).floor() as usize;
            let a0 = d[2][0] / 0.3 * PI;
            let mut dirs = vec![];
            for i in 0..n {
                let a = a0 + PI * 2.0 * i as f64 / n as f64 + d[i / 5 % 4][2 + i % 5 * 2] / 0.15 * 0.28;
                let start = [p[0] + a.cos() * crush, p[1] + a.sin() * crush];
                let reach = if r.next() < 0.22 { r.range(0.07, 0.24) } else { 9.0 };
                let pts = run(&mut r, start, a, reach);
                // a branch now and then, splitting off part way
                if r.next() < 0.38 && pts.len() > 6 {
                    let k = pts.len() * (3 + (r.next() * 3.0) as usize) / 10;
                    let side = if r.next() < 0.5 { -1.0 } else { 1.0 };
                    let turn = a + side * r.range(0.3, 0.6);
                    let reach = if r.next() < 0.4 { r.range(0.05, 0.15) } else { 9.0 };
                    let br = run(&mut r, pts[k], turn, reach);
                    b.lines.push((br, 0.8));
                }
                dirs.push((a, pts.clone()));
                b.lines.push((pts, 1.3));
            }
            // the rings between the radials, near the blow
            let rings = [r.range(0.022, 0.045), r.range(0.065, 0.12)];
            let at = |pts: &Vec<[f64; 2]>, rad: f64| -> Option<[f64; 2]> {
                pts.iter().find(|q| ((q[0] - p[0]).powi(2) + (q[1] - p[1]).powi(2)).sqrt() >= rad).copied()
            };
            // (nearly straight from one radial to the next, each at its own
            // distance, in some sectors only, now and then stopping short)
            for (ri, &rad) in rings.iter().enumerate() {
                let chance = if ri == 0 { 0.5 } else { 0.28 };
                for i in 0..n {
                    let j = (i + 1) % n;
                    if r.next() > chance {
                        continue;
                    }
                    let r0 = rad * r.range(0.75, 1.3);
                    let (Some(q0), Some(q1)) = (at(&dirs[i].1, r0), at(&dirs[j].1, r0 * r.range(0.8, 1.2))) else { continue };
                    let reach = if r.next() < 0.3 { r.range(0.5, 0.8) } else { 1.0 };
                    let q1 = [q0[0] + (q1[0] - q0[0]) * reach, q0[1] + (q1[1] - q0[1]) * reach];
                    let m = [(q0[0] + q1[0]) / 2.0 + r.range(-0.003, 0.003), (q0[1] + q1[1]) / 2.0 + r.range(-0.003, 0.003)];
                    b.lines.push((vec![q0, m, q1], 0.8));
                }
            }
            // a shard gone between two radials
            if d[3][1] > 0.12 {
                let i = ((d[3][2] + 0.15) / 0.3 * n as f64).floor() as usize % n;
                let j = (i + 1) % n;
                let rad = rings[(d[3][3] > 0.0) as usize];
                if let (Some(q0), Some(q1)) = (at(&dirs[i].1, rad), at(&dirs[j].1, rad)) {
                    let (a0, a1) = (dirs[i].0, dirs[j].0);
                    b.hole = vec![
                        [p[0] + a0.cos() * crush, p[1] + a0.sin() * crush],
                        q0,
                        q1,
                        [p[0] + a1.cos() * crush, p[1] + a1.sin() * crush],
                    ];
                }
            }
            // a second, lighter blow
            if d[3][0] > 0.15 {
                let p2 = [(d[2][1] * 0.5).clamp(-hw + 0.06, hw - 0.06), (-d[1][1] * 0.7).clamp(-hh + 0.06, hh - 0.06)];
                let crush2 = r.range(0.004, 0.008);
                b.hits.push((p2, crush2));
                let n2 = 4 + (r.next() * 3.0) as usize;
                let a2 = r.range(0.0, PI * 2.0);
                for i in 0..n2 {
                    let a = a2 + PI * 2.0 * i as f64 / n2 as f64 + r.range(-0.3, 0.3);
                    let start = [p2[0] + a.cos() * crush2, p2[1] + a.sin() * crush2];
                    let reach = if r.next() < 0.6 { r.range(0.04, 0.16) } else { 9.0 };
                    b.lines.push((run(&mut r, start, a, reach), 1.0));
                }
            }
        } else {
            // a crack run across it from the bottom rail (a slammed door, the hanger)
            let x0 = d[0][0] / 0.3 * (hw - 0.05);
            let up = PI / 2.0 + (d[0][1] / 0.5) * 0.75;
            let pts = run(&mut r, [x0, -hh - 0.003], up, 9.0);
            let k = pts.len() / 2;
            if k > 2 {
                let side = if d[2][0] < 0.0 { -1.0 } else { 1.0 };
                let turn = up + side * r.range(0.35, 0.7);
                let br = run(&mut r, pts[k], turn, 9.0);
                b.lines.push((br, 0.9));
            }
            b.lines.push((pts, 1.1));
            if d[3][0] > 0.0 {
                let y0 = d[1][1] * 0.6;
                let side = if d[3][2] < 0.0 { -1.0 } else { 1.0 };
                let a = if side < 0.0 { r.range(-0.4, 0.4) } else { PI + r.range(-0.4, 0.4) };
                let reach = r.range(0.12, 0.35);
                let pts = run(&mut r, [side * (hw + 0.003), y0], a, reach);
                b.lines.push((pts, 0.9));
            }
        }
        b
    }
}

/// The mirror's glass: silver behind it (`Patch::Glass`, nearly all of the
/// light back), its map where the silver has gone to the black paint
fn mirror_glass_mat(b: &MirrorBreak) -> Mat {
    let m = phong!({ color: 0xffffff, specular: 0x2a2a2a, shininess: 4000, transparent: true, depthWrite: false });
    {
        let mut mm = m.m();
        mm.map = Some(mirror_silver_tex(b));
        mm.patch = Patch::Glass;
        mm.look_settled = true;
        mm.name = "mirror_glass".into();
    }
    {
        let pu = m.r().patch_uniforms.clone();
        let f = &mut pu.borrow_mut().f;
        // the room in it (the light where it hangs, off the walls and floor
        // it faces: a surface of about a third albedo), and nothing behind it
        f[0] = 0.13;
        // no mist
        f[4] = 0.5;
        f[5] = 0.6;
        f[6] = 0.3;
        f[7] = 0.0;
        // the map's grey is the silver gone, to the black paint behind (linear)
        f[8] = 1.0;
        f[9] = 0.016;
        f[10] = 0.015;
        f[11] = 0.013;
        f[12] = 0.0;
        // silvered: most of the light comes back at any angle
        f[13] = 0.86;
    }
    m
}

/// Where the silver has gone (grey: how much, to the black paint behind):
/// the black edge, a band creeping in from the frame with a wandering,
/// feathered front (deepest along the bottom, where the cleaning ran down,
/// and in its corners), spots of it near the front, pinholes; the silver
/// flaked along the cracks and round the blow. And, faintly, the darker
/// room low in it (the floor and the lower walls it gives back)
fn mirror_silver_tex(b: &MirrorBreak) -> Tex {
    let b = b.clone();
    canvas_tex_opt(384, 578, false, move |ctx, w, h| {
        let (gw, gh) = (MIRROR_GLASS[0] + MIRROR_UNDER, MIRROR_GLASS[1] + MIRROR_UNDER);
        let px = |p: [f64; 2]| ((p[0] / gw + 0.5) * w, (0.5 - p[1] / gh) * h);
        ctx.set_fill_style("#000");
        ctx.fill_rect(0, 0, w, h);
        // the room in it, darker low down
        let gr = ctx.create_linear_gradient(0, 0, 0, h);
        gr.add_color_stop(0, "rgba(255,255,255,0)");
        gr.add_color_stop(0.4, "rgba(255,255,255,0.06)");
        gr.add_color_stop(1, "rgba(255,255,255,0.3)");
        ctx.set_fill_style(&gr);
        ctx.fill_rect(0, 0, w, h);
        let spot = |ctx: &mut Canvas, x: f64, y: f64, r: f64, a: f64| {
            let stops = [
                (0.0, format!("rgba(255,255,255,{a:.3})")),
                (0.5, format!("rgba(255,255,255,{:.3})", a * 0.75)),
                (1.0, "rgba(255,255,255,0)".into()),
            ];
            mirror_blob(ctx, x, y, r, 0.55 + 0.45 * rnd(), rnd() * 3.0, &stops);
        };
        // the black edge along each side: (how deep it runs in px at most, along the
        // side's length in px); its front wanders (three waves of their own) and
        // runs deeper into the corners
        let sides: [(f64, f64); 4] = [(16.0, w), (7.0, h), (7.0, h), (4.0, w)]; // bottom, left, right, top
        for (si, &(deep, len)) in sides.iter().enumerate() {
            let ph = [rnd() * 6.3, rnd() * 6.3, rnd() * 6.3];
            let fr = [2.0 + rnd() * 2.0, 6.0 + rnd() * 5.0, 15.0 + rnd() * 10.0];
            let depth = |t: f64| -> f64 {
                let wave = 0.5
                    + 0.22 * (fr[0] * t * 6.28 + ph[0]).sin()
                    + 0.16 * (fr[1] * t * 6.28 + ph[1]).sin()
                    + 0.1 * (fr[2] * t * 6.28 + ph[2]).sin();
                // in stretches, not all along: where the wave is low the silver held
                let on = ((wave - 0.38) / 0.4).clamp(0.0, 1.0);
                let corner = 1.0
                    + 1.6 * ((1.0 - (t * 2.0 - 1.0).abs()) * -9.0).exp()
                    + if si == 0 { 0.0 } else { 1.5 * (-t * 7.0).exp() * (si != 3) as u8 as f64 };
                deep * (on * on * (3.0 - 2.0 * on)) * wave * corner
            };
            // where on the canvas a point `d` px in from the side, `t` along it, is
            let at = |t: f64, d: f64| -> (f64, f64) {
                match si {
                    0 => (t * w, h - d),
                    1 => (d, h - t * h),
                    2 => (w - d, h - t * h),
                    _ => (t * w, d),
                }
            };
            ctx.set_fill_style("rgba(255,255,255,0.85)");
            ctx.begin_path();
            let (x0, y0) = at(0.0, 0.0);
            ctx.move_to(x0, y0);
            let n = (len / 3.0) as usize;
            for k in 0..=n {
                let t = k as f64 / n as f64;
                let (x, y) = at(t, depth(t));
                ctx.line_to(x, y);
            }
            let (x1, y1) = at(1.0, 0.0);
            ctx.line_to(x1, y1);
            ctx.close_path();
            ctx.fill();
            // the front feathered, and spots of it ahead of the front
            for _ in 0..(len / 2.5) as usize {
                let t = rnd();
                let d = depth(t);
                if d < 0.5 {
                    continue;
                }
                let (x, y) = at(t, d * (0.8 + 0.4 * rnd()));
                spot(ctx, x, y, 1.0 + rnd() * d.min(10.0) * 0.45, 0.3 + 0.4 * rnd());
            }
            for _ in 0..(len / 6.0) as usize {
                let t = rnd();
                let d = depth(t).max(1.5) * (1.3 + 2.5 * rnd().powi(2));
                let (x, y) = at(t, d);
                spot(ctx, x, y, 0.6 + rnd() * rnd() * 3.0, 0.4 + 0.5 * rnd());
            }
        }
        // pinholes, all over but most low down
        for _ in 0..90 {
            let (x, y) = (rnd() * w, h * rnd().powf(0.6));
            ctx.set_fill_style(format!("rgba(255,255,255,{:.2})", 0.5 + 0.5 * rnd()));
            ctx.begin_path();
            ctx.arc(x, y, 0.35 + rnd() * rnd() * 1.2, 0, 7);
            ctx.fill();
        }
        // along the cracks, the silver flaked; round the blow more of it
        ctx.set_line_cap("round");
        ctx.set_line_join("round");
        for (pts, k) in &b.lines {
            ctx.set_stroke_style("rgba(255,255,255,0.55)");
            ctx.set_line_width(0.9 * k);
            ctx.begin_path();
            for (i, p) in pts.iter().enumerate() {
                let (x, y) = px(*p);
                if i == 0 {
                    ctx.move_to(x, y);
                } else {
                    ctx.line_to(x, y);
                }
            }
            ctx.stroke();
        }
        for (p, cr) in &b.hits {
            let (x, y) = px(*p);
            let rr = cr / gw * w;
            for _ in 0..10 {
                let (a, d) = (rnd() * 7.0, rr * (1.0 + 2.5 * rnd()));
                spot(ctx, x + a.cos() * d, y + a.sin() * d, 1.0 + rnd() * rr * 0.8, 0.4 + 0.4 * rnd());
            }
        }
    })
}

/// A soft blob painted on a canvas: a radial fade of radius `r` squashed to
/// `k` of it across and turned by `turn`, the canvas squashed so the fade is
/// never cut off by its outline
fn mirror_blob(ctx: &mut Canvas, x: f64, y: f64, r: f64, k: f64, turn: f64, stops: &[(f64, String)]) {
    ctx.save();
    ctx.translate(x, y);
    ctx.rotate(turn);
    ctx.scale(1.0, k);
    let g = ctx.create_radial_gradient(0, 0, 0, 0, 0, r);
    for (at, c) in stops {
        g.add_color_stop(*at, c);
    }
    ctx.set_fill_style(&g);
    ctx.begin_path();
    ctx.arc(0, 0, r, 0, 7);
    ctx.fill();
    ctx.restore();
}

/// The mirror's face: a film of dust, dried splashes and drips on its lower
/// half, a wiped smear, and the breakage (each crack a dark line where the
/// silver broke and a bright one where its edge catches the light, the
/// crushed spot of the blow, a shard gone to the steel back); clear elsewhere
fn mirror_film_tex(b: &MirrorBreak) -> Tex {
    let b = b.clone();
    canvas_tex_opt(512, 778, false, move |ctx, w, h| {
        let (gw, gh) = (MIRROR_GLASS[0], MIRROR_GLASS[1]);
        let px = |p: [f64; 2]| ((p[0] / gw + 0.5) * w, (0.5 - p[1] / gh) * h);
        ctx.clear_rect(0, 0, w, h);
        // the frame's lip seen again in the glass, round its edge, the glass's
        // thickness a dark line between
        ctx.set_fill_style("rgba(150,152,150,0.42)");
        for (x, y, rw, rh) in [(0.0, 0.0, w, 3.0), (0.0, h - 3.0, w, 3.0), (0.0, 0.0, 3.0, h), (w - 3.0, 0.0, 3.0, h)] {
            ctx.fill_rect(x, y, rw, rh);
        }
        ctx.set_stroke_style("rgba(18,18,17,0.45)");
        ctx.set_line_width(1.2);
        ctx.stroke_rect(3.6, 3.6, w - 7.2, h - 7.2);
        // dust, unevenly
        for _ in 0..16 {
            let (x, y, r) = (rnd() * w, rnd() * h, w * (0.1 + 0.25 * rnd()));
            let g = ctx.create_radial_gradient(x, y, 0, x, y, r);
            g.add_color_stop(0, &format!("rgba(176,172,162,{:.3})", 0.04 + 0.05 * rnd()));
            g.add_color_stop(1, "rgba(176,172,162,0)");
            ctx.set_fill_style(&g);
            ctx.fill_rect(0, 0, w, h);
        }
        // a wiped smear, an arc of a cloth or a palm
        ctx.set_line_cap("round");
        for _ in 0..3 {
            let (x, y, r) = (w * (0.2 + 0.6 * rnd()), h * (0.3 + 0.5 * rnd()), w * (0.12 + 0.2 * rnd()));
            ctx.set_stroke_style(format!("rgba(186,182,172,{:.3})", 0.04 + 0.04 * rnd()));
            ctx.set_line_width(8.0 + 14.0 * rnd());
            ctx.begin_path();
            let a = rnd() * 6.3;
            ctx.arc(x, y, r, a, a + 0.8 + rnd() * 1.4);
            ctx.stroke();
        }
        // dried splashes on the lower half, a few run down
        for _ in 0..170 {
            let x = rnd() * w;
            let y = h * (0.45 + 0.55 * rnd().powf(0.6));
            let r = 0.5 + rnd() * rnd() * 2.6;
            let a = 0.12 + 0.28 * rnd();
            ctx.set_fill_style(format!("rgba(206,203,194,{a:.3})"));
            ctx.begin_path();
            ctx.ellipse(x, y, r, r * (0.7 + 0.3 * rnd()), rnd() * 3.0, 0, 7);
            ctx.fill();
            if rnd() < 0.08 {
                ctx.set_stroke_style(format!("rgba(200,197,188,{:.3})", a * 0.5));
                ctx.set_line_width(r * 0.9);
                ctx.begin_path();
                ctx.move_to(x, y);
                ctx.line_to(x + rnd() * 2.0 - 1.0, y + 10.0 + rnd() * 50.0);
                ctx.stroke();
            }
        }
        // the shard gone: the galvanised back, its paint left in patches
        if b.hole.len() > 2 {
            ctx.save();
            ctx.begin_path();
            for (i, p) in b.hole.iter().enumerate() {
                let (x, y) = px(*p);
                if i == 0 {
                    ctx.move_to(x, y);
                } else {
                    ctx.line_to(x, y);
                }
            }
            ctx.close_path();
            ctx.set_fill_style("#3f403f");
            ctx.fill();
            ctx.clip();
            for _ in 0..40 {
                let (x, y) = px(b.hole[(rnd() * b.hole.len() as f64) as usize % b.hole.len()]);
                ctx.set_fill_style(format!("rgba(22,21,20,{:.2})", 0.4 + 0.5 * rnd()));
                ctx.begin_path();
                ctx.ellipse(x + rnd() * 30.0 - 15.0, y + rnd() * 30.0 - 15.0, 2.0 + rnd() * 9.0, 2.0 + rnd() * 6.0, rnd() * 3.0, 0, 7);
                ctx.fill();
            }
            ctx.restore();
        }
        // the cracks: dark where the silver broke, bright where the edge catches the light
        ctx.set_line_join("round");
        let path = |ctx: &mut Canvas, pts: &[[f64; 2]], dx: f64, dy: f64| {
            ctx.begin_path();
            for (i, p) in pts.iter().enumerate() {
                let (x, y) = px(*p);
                if i == 0 {
                    ctx.move_to(x + dx, y + dy);
                } else {
                    ctx.line_to(x + dx, y + dy);
                }
            }
            ctx.stroke();
        };
        for (pts, k) in &b.lines {
            ctx.set_stroke_style("rgba(14,13,12,0.85)");
            ctx.set_line_width(1.9 * k);
            path(ctx, pts, 0.0, 0.0);
            ctx.set_stroke_style(format!("rgba(236,234,228,{:.2})", 0.6 + 0.25 * rnd()));
            ctx.set_line_width(1.1 * k);
            path(ctx, pts, 0.9, -0.8);
        }
        if b.hole.len() > 2 {
            let mut edge = b.hole.clone();
            edge.push(b.hole[0]);
            ctx.set_stroke_style("rgba(236,234,228,0.8)");
            ctx.set_line_width(1.2);
            path(ctx, &edge, 0.0, 0.0);
        }
        // the crushed spot: glass ground to white, splinters out of it
        for (p, cr) in &b.hits {
            let (x, y) = px(*p);
            let rr = cr / gw * w;
            ctx.set_fill_style("rgba(214,212,204,0.85)");
            ctx.begin_path();
            for i in 0..14 {
                let a = i as f64 / 14.0 * PI * 2.0;
                let q = rr * (0.6 + 0.6 * rnd());
                if i == 0 {
                    ctx.move_to(x + a.cos() * q, y + a.sin() * q);
                } else {
                    ctx.line_to(x + a.cos() * q, y + a.sin() * q);
                }
            }
            ctx.close_path();
            ctx.fill();
            for _ in 0..34 {
                let a = rnd() * 7.0;
                let (r0, r1) = (rr * rnd(), rr * (1.0 + 1.6 * rnd()));
                let v = if rnd() < 0.5 { 22 } else { 226 };
                ctx.set_stroke_style(format!("rgba({v},{v},{},0.7)", v - 6));
                ctx.set_line_width(0.6 + 0.5 * rnd());
                ctx.begin_path();
                ctx.move_to(x + a.cos() * r0, y + a.sin() * r0);
                ctx.line_to(x + a.cos() * r1, y + a.sin() * r1);
                ctx.stroke();
            }
        }
    })
}

/// Second CRT, unlit — decorative variant, more detail
pub fn build_tv_on_floor(g: &mut Game, cx: &mut PropsCtx) -> Built {
    let _ = cx;
    let tg = group();
    // beige plastic body
    let beige_mat = lambert!({ color: 0x8a8270 });
    let body = mesh(pg("Props__crtTv_body", box_geo!(0.85, 0.62, 0.55)), &beige_mat);
    body.position().y = 0.31;
    tg.add(&body);
    // recessed screen bezel
    // narrowed to fit between the speaker grille (x < -0.295) and the control
    // panel (x > 0.265); at 0.72 m it covered the grille and z-fought the panel
    let bezel = mesh(pg("Props__crtTv_bezel", box_geo!(0.72, 0.52, 0.02)), &g.mats.dark);
    bezel.scale().x = 0.764;
    bezel.position().set(-0.015, 0.31, 0.278);
    tg.add(&bezel);
    let screen = mesh(plane_geo!(0.66, 0.46), basic_mat!({ color: 0x1a1a1a }));
    screen.scale().x = 0.742;
    screen.position().set(-0.015, 0.31, 0.29);
    tg.add(&screen);
    // reflection highlight on screen (subtle)
    let highlight = mesh(plane_geo!(0.15, 0.22), basic_mat!({ color: 0x3a3a44, transparent: true, opacity: 0.35 }));
    highlight.position().set(-0.18, 0.42, 0.291);
    tg.add(&highlight);
    // control panel on right side
    let panel = mesh(pg("Props__crtTv_panel", box_geo!(0.15, 0.5, 0.02)), lambert!({ color: 0x6a6250 }));
    panel.position().set(0.34, 0.31, 0.278);
    tg.add(&panel);
    // knobs
    for k in 0..3 {
        let kf = k as f64;
        let knob = mesh(pg("Props__crtTv_knob", cyl_geo!(0.028, 0.032, 0.03, 10)), &g.mats.chrome);
        knob.rotation().x = PI / 2.0;
        knob.position().set(0.34, 0.48 - kf * 0.09, 0.29);
        tg.add(&knob);
        let tick = mesh(box_geo!(0.02, 0.005, 0.005), &g.mats.dark);
        tick.position().set(0.34, 0.48 - kf * 0.09, 0.31);
        tg.add(&tick);
    }
    // channel display (analog)
    let ch_disp = mesh(pg("Props__crtTv_channelWindow", box_geo!(0.06, 0.03, 0.02)), lambert!({ color: 0x1a1a1a }));
    ch_disp.position().set(0.34, 0.15, 0.29);
    tg.add(&ch_disp);
    // speaker grille
    let grille = mesh(plane_geo!(0.09, 0.4), lambert!({ color: 0x2a2620 }));
    grille.position().set(-0.34, 0.31, 0.279);
    tg.add(&grille);
    for s in 0..8 {
        let slot = mesh(pg("Props__crtTv_grilleSlot", box_geo!(0.06, 0.008, 0.01)), &g.mats.dark);
        slot.position().set(-0.34, 0.5 - s as f64 * 0.05, 0.28);
        tg.add(&slot);
    }
    // rabbit-ears antenna (bent)
    for s in [-1.0, 1.0] {
        let ant = mesh(pg("Props__crtTv_antenna", cyl_geo!(0.006, 0.006, 0.4, 6)), &g.mats.chrome);
        ant.position().set(s * 0.1, 0.75, -0.1);
        ant.rotation().z = s * 0.6;
        ant.rotation().x = -0.3;
        tg.add(&ant);
    }
    // power cord curling behind
    let cord = mesh(cyl_geo!(0.006, 0.006, 0.55, 6), &g.mats.dark);
    cord.position().set(0.28, 0.05, -0.24);
    cord.rotation().z = 0.3;
    tg.add(&cord);
    Built::new(tg, 0.9)
}

pub fn build_book_stack(g: &mut Game, cx: &mut PropsCtx) -> Built {
    let _ = (g, cx);
    let bg = group();
    let colors = [0x6a3a2au32, 0x2a5a6a, 0x5a4a2a, 0x4a2a4a, 0x2a4a2a, 0x8a4020, 0x203a5a, 0x60553a];
    let mut y = 0.0;
    let n = irand(3, 6);
    for _i in 0..n {
        let w = rand2(0.18, 0.26);
        let h = rand2(0.035, 0.055);
        let d = rand2(0.14, 0.2);
        let color = choice(&colors);
        let book = mesh(pg("Props__book_cover", box_geo!(w, h, d)), lambert!({ color: color }));
        let px = rand2(-0.03, 0.03);
        let pz = rand2(-0.03, 0.03);
        book.position().set(px, y + h / 2.0, pz);
        book.rotation().y = rand2(-0.3, 0.3);
        // page stripe on the side (paper interior)
        let pages = mesh(pg("Props__book_pages", box_geo!(w - 0.01, h * 0.7, d - 0.005)), lambert!({ color: 0xe8ddc0 }));
        pages.position().y = 0.001;
        book.add(&pages);
        // spine text (tiny darker rectangle)
        let spine_text = mesh(box_geo!(w * 0.5, h * 0.5, 0.003), lambert!({ color: 0xd8c880 }));
        spine_text.position().set(0, 0, d / 2.0 + 0.001);
        book.add(&spine_text);
        // spine text on other side too
        let spine_text2 = mesh(box_geo!(w * 0.5, h * 0.5, 0.003), lambert!({ color: 0xd8c880 }));
        spine_text2.position().set(0, 0, -d / 2.0 - 0.001);
        book.add(&spine_text2);
        // occasional bookmark ribbon
        if rnd() < 0.25 {
            let ribbon = mesh(pg("Props__book_ribbon", box_geo!(0.015, h + 0.06, 0.005)), lambert!({ color: 0xa02020 }));
            ribbon.position().set(rand2(-0.05, 0.05), 0.02, d / 2.0 - 0.005);
            book.add(&ribbon);
        }
        bg.add(&book);
        y += h;
    }
    // occasional standing book on top, leaning
    if rnd() < 0.5 {
        let standing = mesh(pg("Props__book_standing", box_geo!(0.16, 0.22, 0.035)), lambert!({ color: choice(&colors) }));
        let px = rand2(-0.06, 0.06);
        let pz = rand2(-0.06, 0.06);
        standing.position().set(px, y + 0.11, pz);
        standing.rotation().z = rand2(-0.5, 0.5);
        standing.rotation().y = rand2(0.0, PI * 2.0);
        bg.add(&standing);
        y += 0.24;
    }
    Built::new(bg, y)
}

pub fn build_coat_rack(g: &mut Game, cx: &mut PropsCtx) -> Built {
    let _ = cx;
    let rg = group();
    let pole = mesh(pg("Props__coatRack_pole", cyl_geo!(0.025, 0.025, 1.7, 8)), &g.mats.wood);
    pole.position().y = 0.85;
    rg.add(&pole);
    let base = mesh(pg("Props__coatRack_base", cyl_geo!(0.22, 0.22, 0.04, 12)), &g.mats.wood);
    base.position().y = 0.02;
    rg.add(&base);
    for i in 0..4 {
        let hook = mesh(pg("Props__coatRack_hook", torus_geo!(0.06, 0.012, 4, 8)), &g.mats.chrome);
        // one upright loop per side, sticking out radially from the pole (its
        // near side in the pole); centred on the pole they coincided in pairs
        let ha = i as f64 * (PI / 2.0);
        hook.position().set(ha.cos() * 0.079, 1.6, ha.sin() * 0.079);
        hook.rotation().y = -ha;
        rg.add(&hook);
    }
    // hanging coat: on the +z hook, its back against the pole (it was pierced by it)
    if rnd() < 0.7 {
        let coat = mesh(pg("Props__coatRack_coat", box_geo!(0.4, 0.7, 0.1)), lambert!({ color: 0x2a2a2a }));
        coat.position().set(0, 1.18, 0.085);
        rg.add(&coat);
    }
    Built::new(rg, 1.7)
}

/// A dropped tube on the floor
pub fn build_fluorescent_tube(g: &mut Game, cx: &mut PropsCtx) -> Built {
    let _ = cx;
    let tg = group();
    let tube = mesh(cyl_geo!(0.04, 0.04, 0.9, 8), basic_mat!({ color: 0xf3e9bd }));
    tube.rotation().z = PI / 2.0;
    tube.position().y = 0.04;
    tg.add(&tube);
    let end1 = mesh(pg("Props__tube_endCap", cyl_geo!(0.05, 0.05, 0.06, 8)), &g.mats.chrome);
    end1.rotation().z = PI / 2.0;
    end1.position().set(-0.45, 0.04, 0);
    tg.add(&end1);
    let end2 = end1.duplicate();
    end2.position().x = 0.45;
    tg.add(&end2);
    Built::new(tg, 0.1)
}

/// A junk colour (cardboard, cloth, plastic, rust, grey) from one drawn
/// number: which of them from its low bits, how light (`lo`..`hi` of the
/// colour, sRGB) from the rest
fn junk_colour(n: i64, lo: f64, hi: f64) -> u32 {
    const JUNK: [(f64, f64, f64); 6] =
        [(0.62, 0.48, 0.32), (0.45, 0.42, 0.40), (0.38, 0.30, 0.24), (0.52, 0.30, 0.20), (0.30, 0.34, 0.36), (0.55, 0.52, 0.44)];
    let (r, g, b) = JUNK[(n.rem_euclid(6)) as usize];
    let k = lo + (hi - lo) * ((n / 6).rem_euclid(1000) as f64 / 999.0);
    let c = |v: f64| ((v * k / 0.62).clamp(0.0, 1.0) * 255.0).round() as u32;
    (c(r) << 16) | (c(g) << 8) | c(b)
}

/// Absurd amalgam of small junk
pub fn build_pile(g: &mut Game, cx: &mut PropsCtx) -> Built {
    let _ = (g, cx);
    let pg_ = group();
    // the bound `irand(4, 8)` is deliberately redrawn every pass
    let mut i = 0i64;
    while i < irand(4, 8) {
        // (each colour from one draw, as it always was, but a junk colour:
        // a random hex between two had come out any colour at all, lime and
        // orange among them)
        let block = || {
            mesh(
                pg("Props__pile_block", box_geo!(rand2(0.1, 0.3), rand2(0.05, 0.2), rand2(0.1, 0.3))),
                lambert!({ color: junk_colour(irand(0x111111, 0xaaaaaa), 0.10, 0.62) }),
            )
        };
        let lump = || {
            mesh(
                pg("Props__pile_lump", sphere_geo!(rand2(0.06, 0.12), 6, 4)),
                lambert!({ color: junk_colour(irand(0x222222, 0xbb9955), 0.14, 0.70) }),
            )
        };
        let shape = if choice(&[0, 1]) == 0 { block() } else { lump() };
        let (x, y, z) = (rand2(-0.3, 0.3), rand2(0.05, 0.35), rand2(-0.3, 0.3));
        shape.position().set(x, y, z);
        let (a, b, c) = (rand2(0.0, 7.0), rand2(0.0, 7.0), rand2(0.0, 7.0));
        shape.rotation().set(a, b, c);
        pg_.add(&shape);
        i += 1;
    }
    Built::new(pg_, 0.4)
}

/// A curved leaf (a flat shape geometry
/// pushed out in z by a sine of its height) with its pivot at the base.
pub fn make_leaf_shape(len: f64, wid: f64, color: u32, curl: f64) -> Obj {
    // 8-point ellipse-ish leaf shape
    let mut shape = Shape::new();
    shape.move_to(0, 0);
    shape.bezier_curve_to(wid, len * 0.25, wid * 0.6, len * 0.75, 0, len);
    shape.bezier_curve_to(-wid * 0.6, len * 0.75, -wid, len * 0.25, 0, 0);
    let geo = shape_geo!(shape, 12);
    // add gentle curl by pushing z per y
    {
        let mut gg = geo.g();
        let n = gg.vertex_count();
        for i in 0..n {
            let y = gg.position[i * 3 + 1] as f64;
            gg.position[i * 3 + 2] = (((y / len) * PI).sin() * curl * len) as f32;
        }
        gg.touch();
    }
    geo.compute_vertex_normals();
    mesh(geo, lambert!({ color: color, side: DoubleSide }))
}

/// Pot + soil + fronds
pub fn build_potted_plant(g: &mut Game, cx: &mut PropsCtx) -> Built {
    let _ = (g, cx);
    let pp = group();
    // terracotta pot with rim + drainage saucer
    let saucer_mat = lambert!({ color: 0x6a3a22 });
    let saucer = mesh(pg("Props__pottedPlant_saucer", cyl_geo!(0.19, 0.17, 0.02, 14)), &saucer_mat);
    saucer.position().y = 0.01;
    pp.add(&saucer);
    let pot_mat = lambert!({ color: 0x8a4a2a });
    let rim_mat = lambert!({ color: 0x9a5230 });
    let pot = mesh(pg("Props__pottedPlant_pot", cyl_geo!(0.16, 0.12, 0.28, 14)), &pot_mat);
    pot.position().y = 0.16;
    pp.add(&pot);
    // vertical seam lines on pot (hand-thrown look)
    for s in 0..6 {
        let seam = mesh(box_geo!(0.005, 0.24, 0.005), lambert!({ color: 0x5a2f18 }));
        let a = s as f64 * PI / 3.0;
        // on the tapered wall (r 0.12 at the foot .. 0.16 at the top): radius at
        // mid-height, leaning out with the wall; vertical at r 0.155 it stood 3 cm off
        seam.set_rotation_order(EulerOrder::YXZ);
        seam.rotation().set(0, -a, -0.142);
        seam.position().set(a.cos() * 0.1415, 0.16, a.sin() * 0.1415);
        pp.add(&seam);
    }
    let rim = mesh(pg("Props__pottedPlant_rim", cyl_geo!(0.17, 0.17, 0.03, 14)), &rim_mat);
    rim.position().y = 0.30;
    pp.add(&rim);
    // soil + tiny pebbles
    let soil = mesh(pg("Props__pottedPlant_soil", cyl_geo!(0.16, 0.16, 0.02, 12)), lambert!({ color: 0x2a1a10 }));
    soil.position().y = 0.315;
    pp.add(&soil);
    for _i in 0..5 {
        let pebble = mesh(pg("Props__pottedPlant_pebble", sphere_geo!(rand2(0.008, 0.014), 5, 4)), lambert!({ color: 0x5a4030 }));
        let x = rand2(-0.1, 0.1);
        let z = rand2(-0.1, 0.1);
        pebble.position().set(x, 0.33, z);
        pp.add(&pebble);
    }
    // fronds — mix of long thin blades + broader curled leaves
    let leaf_colors = [0x2a5a2au32, 0x3a6a30, 0x25502a, 0x487a3a, 0x1a3a1a];
    let blade_count = irand(10, 16);
    for i in 0..blade_count {
        let a = (i as f64 / blade_count as f64) * PI * 2.0 + rand2(-0.15, 0.15);
        let height = rand2(0.28, 0.55);
        let blade = mesh(pg_own("Props__pottedPlant_blade", box_geo!(0.022, height, 0.005)), lambert!({ color: choice(&leaf_colors) }));
        let x = a.cos() * rand2(0.02, 0.09);
        let z = a.sin() * rand2(0.02, 0.09);
        blade.position().set(x, 0.32 + height / 2.0 - 0.02, z);
        // splay outward from center
        let rx = a.sin() * rand2(0.15, 0.4);
        let rz = -a.cos() * rand2(0.15, 0.4);
        blade.rotation().set(rx, a, rz);
        // taper by scaling top thinner
        blade.geo().translate(0, 0, 0);
        pp.add(&blade);
    }
    // curled broad leaves scattered on top for variety
    let broad_count = irand(3, 5);
    for _i in 0..broad_count {
        let a = rand2(0.0, PI * 2.0);
        let (len, wid, color, curl) = (rand2(0.12, 0.20), rand2(0.05, 0.08), choice(&leaf_colors), rand2(0.12, 0.25));
        let leaf = make_leaf_shape(len, wid, color, curl);
        let x = a.cos() * rand2(0.02, 0.08);
        let z = a.sin() * rand2(0.02, 0.08);
        leaf.position().set(x, 0.33, z);
        let rx = rand2(0.2, 0.6);
        let rz = rand2(-0.3, 0.3);
        leaf.rotation().set(rx, a, rz);
        pp.add(&leaf);
    }
    // dead brown leaves on saucer and floor
    // the bound `irand(2, 4)` is deliberately redrawn every pass
    let mut i = 0i64;
    while i < irand(2, 4) {
        let dead = mesh(pg("Props__plant_deadLeaf", box_geo!(rand2(0.04, 0.06), 0.002, rand2(0.03, 0.05))), lambert!({ color: 0x5a3a20 }));
        let x = rand2(-0.22, 0.22);
        let z = rand2(-0.22, 0.22);
        dead.position().set(x, 0.006, z);
        dead.rotation().y = rand2(0.0, 7.0);
        dead.rotation().x = rand2(-0.15, 0.15);
        pp.add(&dead);
        i += 1;
    }
    // occasional small yellow flower
    if rnd() < 0.25 {
        let f_center = mesh(pg("Props__pottedPlant_flowerCenter", sphere_geo!(0.015, 6, 5)), lambert!({ color: 0xf0c040 }));
        let (x, y, z) = (rand2(-0.05, 0.05), rand2(0.5, 0.7), rand2(-0.05, 0.05));
        f_center.position().set(x, y, z);
        pp.add(&f_center);
        let fc = f_center.pos();
        for p in 0..5 {
            let petal = mesh(pg("Props__pottedPlant_petal", box_geo!(0.02, 0.008, 0.03)), lambert!({ color: 0xf8e0a0 }));
            let pa = (p as f64 / 5.0) * PI * 2.0;
            petal.position().set(fc.x + pa.cos() * 0.02, fc.y, fc.z + pa.sin() * 0.02);
            petal.rotation().y = PI / 2.0 - pa; // long (z) axis radial, like its offset
            pp.add(&petal);
        }
    }
    Built::new(pp, 0.7)
}

/// Large floor plant: a neglected office dracaena (Blender batch `r05`)
pub fn build_floor_plant(g: &mut Game, cx: &mut PropsCtx) -> Built {
    let _ = (g, cx);
    // A Dracaena fragrans on three staggered canes in a 16 in moulded plastic
    // planter on its saucer, left for years: the canes' shoots grown leggy,
    // the old leaves yellowing and hanging, a skirt of dead ones under each
    // crown, brown dry tips, the soil shrunk from the pot, leaves dropped
    // round it (docs/PROPS.md). The parts as modelled, the pot centred.
    use crate::assets::props::prop_part_as_modelled;
    let fp = group();
    // the plant turns about its pot (the old first leaf's spin)
    let plant = group();
    fp.add(&plant);
    let put = |name: &str, mat: &Mat| {
        if let Some(geo) = prop_part_as_modelled(name) {
            plant.add(&mesh(geo, mat));
        }
    };
    // the leaves' dust and blotches laid on at their real scale (a repeat every
    // 0.2 m, from each leaf's own facing), not stretched over the part's box
    let put_leaf = |name: &str, mat: &Mat| {
        if let Some(geo) = prop_part_as_modelled(name) {
            plant.add(&mesh(drac_leaf_uv(&geo), mat));
        }
    };
    // the old moss patches' draws (four: a size and a place each)
    for _m in 0..4 {
        let _ = (rand2(0.04, 0.08), rand2(-0.15, 0.15), rand2(-0.15, 0.15));
    }
    // the old eight broad leaves' draws: the first one's spin turns the plant
    let big_leaf_colors = [0x2a5a28u32, 0x357a30, 0x3a6a30, 0x225022];
    let mut spin = 0.0;
    for i in 0..8 {
        let _ = (rand2(0.22, 0.32), rand2(0.10, 0.14), choice(&big_leaf_colors), rand2(0.18, 0.32));
        let _ = (rand2(-0.05, 0.05), rand2(-0.05, 0.05));
        let (_, b, _) = (rand2(-0.4, -0.1), rand2(0.0, PI * 2.0), rand2(-0.3, 0.3));
        if i == 0 {
            spin = b;
        }
    }
    plant.rotation().y = spin;
    // the old two yellowing leaves' draws
    for _i in 0..2 {
        let _ = (rand2(0.16, 0.22), rand2(0.08, 0.11), rand2(0.25, 0.45));
        let _ = (rand2(-0.12, 0.12), rand2(0.6, 1.2), rand2(-0.12, 0.12));
        let _ = (rand2(-1.2, -0.8), rand2(0.0, PI * 2.0), rand2(-0.3, 0.3));
    }
    // the planter: charcoal moulded plastic gone grey with dust, the white
    // crust of hard water where the saucer stood full and where it was
    // over-watered down its side; the saucer black plastic, dry
    put("Real__dracSaucer", &phong!({ color: 0x2a2826, specular: 0x1a1a1a, shininess: 36 }));
    put("Real__dracPot", &phong!({ map: &drac_pot_tex(), specular: 0x1a1a1a, shininess: 24 }));
    put("Real__dracSoil", &lambert!({ map: &drac_soil_tex() }));
    put("Real__dracCanes", &lambert!({ color: 0x5f5546 }));
    put("Real__dracWax", &lambert!({ color: 0x5c2b21 }));
    put("Real__dracShoots", &lambert!({ color: 0x5b5338 }));
    // the leaves: a dark green filmed with dust, their tips dry, the oldest
    // yellow, the dead papery; each blotched as it went
    let tips = lambert!({ map: &drac_leaf_tex([116, 88, 58], 0.2), side: DoubleSide });
    let dead = lambert!({ map: &drac_leaf_tex([112, 88, 62], 0.15), side: DoubleSide });
    put_leaf("Real__dracLeaves", &phong!({ map: &drac_leaf_tex([51, 69, 43], 0.35), specular: 0x14180f, shininess: 16, side: DoubleSide }));
    put_leaf("Real__dracTips", &tips);
    put_leaf("Real__dracYellow", &lambert!({ map: &drac_leaf_tex([126, 116, 70], 0.25), side: DoubleSide }));
    put_leaf("Real__dracYellowTips", &tips);
    put_leaf("Real__dracDead", &dead);
    put_leaf("Real__dracLitter", &dead);
    // dropped leaves on the floor round it, curled and dry (out past the saucer)
    let fallen = prop_part_as_modelled("Real__dracFallen");
    let browns = [0x8a7150u32, 0x9a8458, 0x7a6244];
    let fallen_tex = drac_leaf_tex([255, 255, 255], 0.0);
    for _i in 0..4 {
        let _ = (rand2(0.08, 0.14), rand2(0.05, 0.08));
        let shade = browns[irand(0, 2) as usize];
        let x = rand2(-0.35, 0.35);
        let z = rand2(-0.35, 0.35);
        let turn = rand2(0.0, 7.0);
        let _ = rand2(-0.2, 0.2);
        if let Some(geo) = &fallen {
            let r = (x * x + z * z).sqrt().max(1e-6);
            let out = 0.25 + 0.42 * (r / 0.495).min(1.0);
            let leaf = mesh(drac_leaf_uv(geo), lambert!({ color: shade, map: &fallen_tex, side: DoubleSide }));
            leaf.position().set(x / r * out, 0.0, z / r * out);
            leaf.rotation().y = turn;
            fp.add(&leaf);
        }
    }
    // now and then spider mites' webbing between the top crown's leaves
    if rnd() < 0.4 {
        let _ = (rand2(-0.5, 0.5), rand2(0.0, 7.0), rand2(-0.5, 0.5));
        put("Real__dracWeb", &lambert!({ color: 0xb4b0a6, transparent: true, opacity: 0.45, depthWrite: false }));
    }
    Built::new(fp, 1.9)
}

/// A leaf part with its map laid on at the leaf's own scale: each point's
/// texture place from the two axes across its facing, a repeat every 0.2 m
fn drac_leaf_uv(geo: &Geo) -> Geo {
    let g = geo.duplicate();
    {
        let mut gg = g.g();
        let n = gg.position.len() / 3;
        let mut uv = Vec::with_capacity(n * 2);
        for i in 0..n {
            let (x, y, z) = (gg.position[i * 3], gg.position[i * 3 + 1], gg.position[i * 3 + 2]);
            let (nx, ny, nz) = (gg.normal[i * 3].abs(), gg.normal[i * 3 + 1].abs(), gg.normal[i * 3 + 2].abs());
            let (u, v) = if ny >= nx && ny >= nz {
                (x, z)
            } else if nx >= nz {
                (z, y)
            } else {
                (x, y)
            };
            uv.extend_from_slice(&[u / 0.2, v / 0.2]);
        }
        gg.uv = uv;
        gg.touch();
    }
    g
}

/// A dracaena leaf's surface, repeating: its colour (`base`, sRGB) a little
/// uneven, darker and lighter streaks, brown blotches where it was hurt, and
/// dust (`dust`: how much) settled on it in a grey film and specks
fn drac_leaf_tex(base: [u8; 3], dust: f64) -> Tex {
    canvas_tex(256, 256, move |ctx, w, h| {
        let [r, g, b] = base;
        ctx.set_fill_style(format!("rgb({r},{g},{b})"));
        ctx.fill_rect(0, 0, w, h);
        let shade = |k: f64| -> String {
            let f = |c: u8| ((c as f64 * k).round() as i64).clamp(0, 255);
            format!("{},{},{}", f(r), f(g), f(b))
        };
        // uneven colour: soft patches lighter and darker
        for _ in 0..40 {
            let (x, y, rr) = (rnd() * w, rnd() * h, 10.0 + rnd() * 40.0);
            let k = 0.82 + 0.36 * rnd();
            let stops = [(0.0, format!("rgba({},0.5)", shade(k))), (1.0, format!("rgba({},0)", shade(k)))];
            mirror_blob(ctx, x, y, rr, 0.3 + 0.7 * rnd(), rnd() * 3.0, &stops);
        }
        // fine streaks, the veins running the leaf's length
        ctx.set_line_width(0.8);
        for i in 0..60 {
            let x = (i as f64 + rnd()) / 60.0 * w;
            ctx.set_stroke_style(format!("rgba({},0.35)", shade(if i % 2 == 0 { 0.86 } else { 1.12 })));
            ctx.begin_path();
            ctx.move_to(x, 0);
            ctx.line_to(x + rnd() * 4.0 - 2.0, h);
            ctx.stroke();
        }
        // brown blotches and pin spots where it was hurt or dried
        for _ in 0..7 {
            let (x, y, rr) = (rnd() * w, rnd() * h, 3.0 + rnd() * 9.0);
            let stops = [(0.0, "rgba(92,66,40,0.75)".to_string()), (0.6, "rgba(104,78,48,0.5)".into()), (1.0, "rgba(104,78,48,0)".into())];
            mirror_blob(ctx, x, y, rr, 0.5 + 0.5 * rnd(), rnd() * 3.0, &stops);
        }
        // dust: a grey film, thicker in drifts, and specks
        if dust > 0.0 {
            for _ in 0..24 {
                let (x, y, rr) = (rnd() * w, rnd() * h, 16.0 + rnd() * 50.0);
                let a = dust * (0.4 + 0.6 * rnd());
                let stops = [(0.0, format!("rgba(150,146,134,{a:.3})")), (1.0, "rgba(150,146,134,0)".into())];
                mirror_blob(ctx, x, y, rr, 0.5 + 0.5 * rnd(), rnd() * 3.0, &stops);
            }
            for _ in 0..(900.0 * dust) as usize {
                let (x, y) = (rnd() * w, rnd() * h);
                ctx.set_fill_style(format!("rgba(170,166,154,{:.2})", 0.3 + 0.5 * rnd()));
                ctx.begin_path();
                ctx.arc(x, y, 0.4 + rnd() * rnd() * 1.4, 0, 7);
                ctx.fill();
            }
        }
    })
}

/// The planter's plastic, by height (the part's box: v 0 its foot, 1 its
/// rim): charcoal gone grey with dust, more of it up by the rim; the white
/// crust of hard water in a band at its foot where the saucer stood full,
/// runs of it down from the rim where it was over-watered; scuffs
fn drac_pot_tex() -> Tex {
    canvas_tex(256, 256, |ctx, w, h| {
        ctx.set_fill_style("rgb(44,43,41)");
        ctx.fill_rect(0, 0, w, h);
        let gr = ctx.create_linear_gradient(0, 0, 0, h);
        gr.add_color_stop(0, "rgba(140,136,126,0.38)");
        gr.add_color_stop(0.3, "rgba(140,136,126,0.1)");
        gr.add_color_stop(1, "rgba(140,136,126,0.03)");
        ctx.set_fill_style(&gr);
        ctx.fill_rect(0, 0, w, h);
        // the dust unevenly, in soft patches
        for _ in 0..14 {
            let (x, y, rr) = (rnd() * w, h * rnd(), 12.0 + rnd() * 40.0);
            let stops = [(0.0, format!("rgba(130,126,116,{:.3})", 0.08 + 0.12 * rnd())), (1.0, "rgba(130,126,116,0)".into())];
            mirror_blob(ctx, x, y, rr, 0.4 + 0.6 * rnd(), rnd() * 3.0, &stops);
        }
        // the crust where the saucer's water stood: a band at the foot, its top edge wandering
        let top = h * 0.93;
        ctx.set_fill_style("rgba(196,192,180,0.55)");
        ctx.begin_path();
        ctx.move_to(0, h);
        let ph = rnd() * 6.3;
        for k in 0..=64 {
            let x = k as f64 / 64.0 * w;
            let y = top - 3.0 * (x / w * 6.28 * 3.0 + ph).sin() - rnd() * 3.0;
            ctx.line_to(x, y);
        }
        ctx.line_to(w, h);
        ctx.close_path();
        ctx.fill();
        // its tide line, whiter
        ctx.set_stroke_style("rgba(214,210,198,0.7)");
        ctx.set_line_width(1.5);
        ctx.begin_path();
        for k in 0..=64 {
            let x = k as f64 / 64.0 * w;
            let y = top - 3.0 * (x / w * 6.28 * 3.0 + ph).sin() - rnd() * 1.5;
            if k == 0 {
                ctx.move_to(x, y);
            } else {
                ctx.line_to(x, y);
            }
        }
        ctx.stroke();
        // runs down from the rim
        ctx.set_line_cap("round");
        for _ in 0..9 {
            let x = rnd() * w;
            let len = h * (0.2 + 0.55 * rnd());
            let y0 = h * (0.04 + 0.04 * rnd());
            ctx.set_stroke_style(format!("rgba(200,196,184,{:.2})", 0.2 + 0.3 * rnd()));
            ctx.set_line_width(1.5 + rnd() * 3.5);
            ctx.begin_path();
            ctx.move_to(x, y0);
            ctx.quadratic_curve_to(x + rnd() * 6.0 - 3.0, y0 + len * 0.5, x + rnd() * 8.0 - 4.0, y0 + len);
            ctx.stroke();
        }
        // scuffs, lighter
        ctx.set_line_width(0.8);
        for _ in 0..40 {
            let (x, y) = (rnd() * w, h * (0.3 + 0.7 * rnd()));
            let a = rnd() * 0.6 - 0.3;
            let l = 4.0 + rnd() * 16.0;
            ctx.set_stroke_style(format!("rgba(120,118,112,{:.2})", 0.3 + 0.4 * rnd()));
            ctx.begin_path();
            ctx.move_to(x, y);
            ctx.line_to(x + a.cos() * l, y + a.sin() * l);
            ctx.stroke();
        }
        // dust specks
        for _ in 0..500 {
            let (x, y) = (rnd() * w, h * rnd().powf(1.6));
            ctx.set_fill_style(format!("rgba(160,156,146,{:.2})", 0.25 + 0.4 * rnd()));
            ctx.begin_path();
            ctx.arc(x, y, 0.4 + rnd() * 0.9, 0, 7);
            ctx.fill();
        }
    })
}

/// The potting soil seen from above, dried out: dark peat, crumbs lighter
/// and darker, the white grains of perlite, a few bits of dead leaf
fn drac_soil_tex() -> Tex {
    canvas_tex(256, 256, |ctx, w, h| {
        ctx.set_fill_style("rgb(62,48,36)");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..900 {
            let (x, y) = (rnd() * w, rnd() * h);
            let v = if rnd() < 0.5 { (42, 32, 24) } else { (96, 78, 58) };
            ctx.set_fill_style(format!("rgba({},{},{},{:.2})", v.0, v.1, v.2, 0.4 + 0.5 * rnd()));
            ctx.begin_path();
            ctx.arc(x, y, 0.6 + rnd() * rnd() * 3.0, 0, 7);
            ctx.fill();
        }
        // perlite
        for _ in 0..140 {
            let (x, y) = (rnd() * w, rnd() * h);
            ctx.set_fill_style(format!("rgba(206,202,192,{:.2})", 0.7 + 0.3 * rnd()));
            ctx.begin_path();
            ctx.arc(x, y, 0.8 + rnd() * 1.6, 0, 7);
            ctx.fill();
        }
        // bits of dead leaf
        ctx.set_line_cap("round");
        for _ in 0..10 {
            let (x, y, a, l) = (rnd() * w, rnd() * h, rnd() * 6.3, 6.0 + rnd() * 18.0);
            ctx.set_stroke_style("rgba(128,100,66,0.8)");
            ctx.set_line_width(2.0 + rnd() * 3.0);
            ctx.begin_path();
            ctx.move_to(x, y);
            ctx.line_to(x + a.cos() * l, y + a.sin() * l);
            ctx.stroke();
        }
    })
}

/// Two doors set in wall, call button, floor indicator
pub fn build_elevator(g: &mut Game, cx: &mut PropsCtx, functional: bool) -> Built {
    let _ = cx;
    let eg = group();
    // frame around
    let outer = mesh(pg("Props__elevator_frame", box_geo!(1.6, 2.4, 0.15)), lambert!({ color: 0x3a3f45 }));
    outer.position().y = 1.2;
    eg.add(&outer);
    // two doors that meet in the middle
    let door_mat = lambert!({ color: 0x6a7078, emissive: 0x101215 });
    for s in [-1.0, 1.0] {
        let door = mesh(pg("Props__elevator_door", box_geo!(0.7, 2.05, 0.05)), &door_mat);
        door.position().set(s * 0.36, 1.05, 0.08);
        eg.add(&door);
        // subtle seam highlight
        let seam = mesh(box_geo!(0.005, 2.0, 0.01), &g.mats.dark);
        seam.position().set(s * 0.005, 1.05, 0.11);
        eg.add(&seam);
    }
    // floor number indicator (small LED display above doors)
    let display = mesh(pg("Props__elevator_display", box_geo!(0.5, 0.12, 0.02)), lambert!({ color: 0x1a1a1a }));
    display.position().set(0, 2.32, 0.09);
    eg.add(&display);
    let num_text = if functional { fmt_num(irand(1, 9) as f64) } else { "--".to_string() };
    let display_num = mesh(
        plane_geo!(0.44, 0.08),
        basic_mat!({
            map: ttex(&num_text, 128.0, 64.0, "bold 44px \"Courier New\", monospace", if functional { "#f04010" } else { "#603010" }, 6.0),
            transparent: true,
        }),
    );
    display_num.position().set(0, 2.32, 0.10);
    eg.add(&display_num);
    // call button panel to the right
    let panel = mesh(pg("Props__elevator_callPanel", box_geo!(0.15, 0.28, 0.03)), lambert!({ color: 0x2a2f35 }));
    panel.position().set(0.88, 1.4, 0.08);
    eg.add(&panel);
    // up/down buttons (chrome triangles)
    let btn_mat = lambert!({
        color: if functional { 0xf0d090u32 } else { 0x3a3025 },
        emissive: if functional { 0x2a2010u32 } else { 0x000000 },
    });
    for b in 0..2 {
        let btn = mesh(pg("Props__elevator_button", cyl_geo!(0.03, 0.03, 0.01, 12)), &btn_mat);
        btn.rotation().x = PI / 2.0;
        btn.position().set(0.88, 1.5 - b as f64 * 0.15, 0.10);
        eg.add(&btn);
    }
    // out-of-service sign hanging over non-functional elevators
    if !functional {
        let sign = mesh(
            plane_geo!(1.2, 0.28),
            basic_mat!({
                map: handwritten_tex("OUT OF SERVICE", &HandOpts {
                    color: "rgba(160,30,20,0.98)".into(), thickness: 8.0, jitter: 2.2, wobble: 1.6,
                    w: 1024.0, h: 220.0, double_stroke: true, drip: true, ..Default::default()
                }),
                transparent: true,
            }),
        );
        sign.position().set(0, 1.6, 0.11);
        sign.rotation().z = rand2(-0.05, 0.05);
        eg.add(&sign);
    }
    // brushed strip above doors
    let strip = mesh(pg("Props__elevator_strip", box_geo!(1.5, 0.04, 0.03)), &g.mats.chrome);
    strip.position().set(0, 2.13, 0.09);
    eg.add(&strip);
    Built::new(eg, 2.4)
}

/// Rare stack of loose sheets that can be picked up
pub fn build_paper_pile(g: &mut Game, cx: &mut PropsCtx) -> Built {
    let _ = cx;
    let pg_ = group();
    let stack = irand(5, 12);
    for i in 0..stack {
        let w = rand2(0.18, 0.24);
        let h = rand2(0.24, 0.30);
        let sheet = mesh(pg("Props__paper_sheet", box_geo!(w, 0.004, h)), &g.mats.paper);
        let x = rand2(-0.06, 0.06);
        let z = rand2(-0.06, 0.06);
        sheet.position().set(x, 0.002 + i as f64 * 0.005, z);
        sheet.rotation().y = rand2(-0.5, 0.5);
        sheet.rotation().x = rand2(-0.02, 0.02);
        pg_.add(&sheet);
    }
    // one sheet standing on its side (curled up)
    let curl = mesh(pg("Props__paper_curledSheet", box_geo!(0.19, 0.24, 0.005)), &g.mats.paper);
    let x = rand2(-0.15, 0.15);
    let z = rand2(-0.15, 0.15);
    curl.position().set(x, 0.12, z);
    let (a, b, c) = (rand2(-0.4, 0.4), rand2(0.0, 7.0), rand2(-0.2, 0.2));
    curl.rotation().set(a, b, c);
    pg_.add(&curl);
    // small paperclip on top
    let clip = mesh(pg("Props__paper_clip", torus_geo!(0.02, 0.003, 4, 8)), &g.mats.chrome);
    clip.position().set(0, stack as f64 * 0.005 + 0.003, 0); // on the top sheet, not at a fixed 0.08
    clip.rotation().x = PI / 2.0;
    pg_.add(&clip);
    Built::new(pg_, 0.15)
}

/// Dark circular void with jagged rim
pub fn build_floor_hole(g: &mut Game, cx: &mut PropsCtx) -> Built {
    let _ = (g, cx);
    let hg = group();
    // black disc at floor level
    let inner = mesh(circle_geo!(0.85, 20), basic_mat!({ color: 0x000000 }));
    inner.rotation().x = -PI / 2.0;
    inner.position().y = 0.005;
    hg.add(&inner);
    // slightly recessed ring
    let ring_mat = lambert!({ color: 0x1a1712 });
    for i in 0..14 {
        let a = (i as f64 / 14.0) * PI * 2.0;
        let chunk = mesh(pg("Props__floorHole_chunk", box_geo!(0.14, 0.06, 0.14)), &ring_mat);
        chunk.position().set(a.cos() * 0.88, 0.03, a.sin() * 0.88);
        chunk.rotation().y = a;
        hg.add(&chunk);
    }
    // shallow cylinder going down for depth cue
    let shaft = mesh(cyl_geo!(0.82, 0.82, 1.2, 20, 1, true), basic_mat!({ color: 0x000000, side: BackSide }));
    shaft.position().y = -0.6;
    hg.add(&shaft);
    Built::new(hg, 0.0)
}
