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
            let (squash, turn) = (0.6 + 0.4 * rnd(), rnd() * 3.0);
            // squashed as a whole, so the fade reaches its edge
            ctx.save();
            ctx.translate(x, y);
            ctx.rotate(turn);
            ctx.scale(1, squash);
            let gr = ctx.create_radial_gradient(0, 0, 0, 0, 0, r);
            gr.add_color_stop(0, "rgba(20,18,14,0.5)");
            gr.add_color_stop(0.6, "rgba(20,18,14,0.35)");
            gr.add_color_stop(1, "rgba(20,18,14,0)");
            ctx.set_fill_style(gr);
            ctx.begin_path();
            ctx.arc(0, 0, r, 0, 7);
            ctx.fill();
            ctx.restore();
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

/// An old hard-shell suitcase lying open on its back, the lid stood up
/// behind, clothes folded in it and a cardigan pulled half out
pub fn build_suitcase(g: &mut Game, cx: &mut PropsCtx) -> Built {
    let _ = cx;
    // a moulded 1960s case (23 x 16 x 7 in), modelled in the batch `r02`:
    // the shells, their linings and aluminium trim (valance, hinges,
    // latches), the grip; the lid stood open on its hinges, held by its
    // check straps (docs/PROPS.md)
    use crate::assets::props::prop_part_as_modelled;
    const HINGE_Y: f64 = 0.112;
    const HINGE_Z: f64 = -0.209;
    const OPEN: f64 = PI / 2.0 + 0.21;
    // the lining's floor inside
    const FLOOR: f64 = 0.0063;
    // the old draws, call for call: five things in it, each its colour, its
    // size and where it lay
    let mut drawn = [(0usize, 0.0f64, 0.0f64, 0.0f64, 0.0f64); 5];
    for d in drawn.iter_mut() {
        let k = irand(0, 3) as usize;
        let r = rand2(0.04, 0.08);
        let x = rand2(-0.2, 0.2);
        let y = 0.15 + rand2(0.0, 0.05);
        let z = rand2(-0.1, 0.1);
        *d = (k, r, x, y, z);
    }
    let sg = group();
    let put = |parent: &Obj, name: &str, mat: &Mat, x: f64, y: f64, z: f64, yaw: f64| {
        if let Some(geo) = prop_part_as_modelled(name) {
            let m = mesh(geo, mat);
            m.position().set(x, y, z);
            m.rotation().y = yaw;
            parent.add(&m);
        }
    };
    // the shell's colour (the last thing's height said it): saddle tan,
    // oxblood, slate blue, olive grey, all gone dull
    let shells = [0x7a5c40u32, 0x5a3029, 0x4a5560, 0x5c5a48];
    let shell = lambert!({ color: shells[(((drawn[4].3 - 0.15) / 0.05 * 4.0) as usize).min(3)] });
    let lining = lambert!({ map: &suitcase_lining_tex() });
    let trim = &g.mats.chrome;
    let grip = lambert!({ color: 0x2a231e });
    for (name, mat) in [
        ("Real__suitcaseBase", &shell),
        ("Real__suitcaseBaseLining", &lining),
        ("Real__suitcaseBaseTrim", trim),
        ("Real__suitcaseHandle", &grip),
    ] {
        put(&sg, name, mat, 0.0, 0.0, 0.0, 0.0);
    }
    let lid = group();
    lid.position().set(0, HINGE_Y, HINGE_Z);
    lid.rotation().x = -OPEN;
    for (name, mat) in [("Real__suitcaseLid", &shell), ("Real__suitcaseLidLining", &lining), ("Real__suitcaseLidTrim", trim)] {
        put(&lid, name, mat, 0.0, -HINGE_Y, -HINGE_Z, 0.0);
    }
    sg.add(&lid);
    // what is in it, each where it was packed, nudged by its draws: trousers
    // folded at the back, a shirt on them, a jumper at the front, socks in
    // the corner, a cardigan pulled out over the rim (or still folded on the
    // jumper); packed from the left or the right (the first thing's height
    // said which, the last thing's place whether the cardigan was pulled)
    const PALETTES: [[u32; 4]; 5] = [
        [0x38383a, 0x2a2e3a, 0x4a3c2e, 0x5e5a54], // trousers: charcoal, navy, brown, grey
        [0x9a8e76, 0x34463a, 0x5a2c2c, 0x6a6864], // jumper: oatmeal, bottle green, burgundy, grey
        [0xc4beb0, 0x9eaab4, 0xb8ae98, 0xbca49c], // shirt: white, pale blue, cream, faded pink
        [0x2c2a2a, 0x625e5a, 0x2a3040, 0x4a3c30], // socks
        [0x76623e, 0x5a2a2c, 0x2c3446, 0x56626a], // cardigan: mustard, maroon, navy, grey-blue
    ];
    const SLOTS: [(&str, f64, f64, f64); 4] = [
        ("Real__suitcaseTrousers", -0.07, -0.085, 0.0),
        ("Real__suitcaseSweater", 0.12, 0.085, 0.0),
        ("Real__suitcaseShirt", -0.11, -0.08, 0.050),
        ("Real__suitcaseSocks", -0.205, 0.12, 0.0),
    ];
    let flip = if drawn[0].3 > 0.175 { -1.0 } else { 1.0 };
    for (i, &(name, sx, sz, sy)) in SLOTS.iter().enumerate() {
        let (k, r, x, _, z) = drawn[i];
        let cloth = lambert!({ color: PALETTES[i][k] });
        let yaw = (r - 0.06) * 2.0 + if i == 3 { 0.5 } else { 0.0 };
        put(&sg, name, &cloth, flip * (sx + x * 0.06), FLOOR + sy, sz + z * 0.1, flip * yaw);
    }
    let (k, r, x, _, z) = drawn[4];
    let cloth = lambert!({ color: PALETTES[4][k] });
    if z > -0.03 {
        put(&sg, "Real__suitcaseSpill", &cloth, x * 0.04, 0.0, 0.0, 0.0);
    } else {
        put(&sg, "Real__suitcaseSweater", &cloth, flip * (0.11 + x * 0.05), FLOOR + 0.075, 0.08, flip * (0.12 + (r - 0.06) * 2.0));
    }
    Built::new(sg, 0.4)
}

/// The suitcase's lining: rayon gone the colour of weak tea, a faint
/// stripe woven in, water-marked and grubby in its corners
fn suitcase_lining_tex() -> Tex {
    canvas_tex_opt(256, 256, false, |ctx, w, h| {
        ctx.set_fill_style("#8a7c70");
        ctx.fill_rect(0, 0, w, h);
        ctx.set_global_alpha(0.2);
        ctx.set_fill_style("#6e6258");
        let mut x = 0.0;
        while x < w {
            ctx.fill_rect(x, 0, 2, h);
            x += 6.0;
        }
        // tide marks
        for _ in 0..3 {
            let (cx0, cy0, r) = (rnd() * w, rnd() * h, 20.0 + rnd() * 50.0);
            ctx.set_global_alpha(0.25);
            ctx.set_stroke_style("#6e5c48");
            ctx.set_line_width(2.0 + rnd() * 2.0);
            ctx.begin_path();
            ctx.ellipse(cx0, cy0, r, r * (0.6 + 0.4 * rnd()), rnd() * 3.0, 0, 7);
            ctx.stroke();
            ctx.set_global_alpha(0.1);
            ctx.set_fill_style("#7a6a58");
            ctx.fill();
        }
        // grime toward the edges (the corners and the rim)
        for (x0, y0, x1, y1) in [(0.0, 0.0, 0.0, 30.0), (0.0, h, 0.0, h - 30.0), (0.0, 0.0, 30.0, 0.0), (w, 0.0, w - 30.0, 0.0)] {
            let gr = ctx.create_linear_gradient(x0, y0, x1, y1);
            gr.add_color_stop(0, "rgba(60,48,36,0.35)");
            gr.add_color_stop(1, "rgba(60,48,36,0)");
            ctx.set_global_alpha(1);
            ctx.set_fill_style(gr);
            ctx.fill_rect(0, 0, w, h);
        }
        ctx.set_global_alpha(1);
    })
}

/// (`yaw` is unused)
pub fn build_broken_mirror(g: &mut Game, cx: &mut PropsCtx, yaw: f64) -> Built {
    let _ = (g, cx, yaw);
    let mg = group();
    let frame = mesh(pg("Props__mirror_frame", box_geo!(0.9, 1.4, 0.06)), lambert!({ color: 0x2a1a10 }));
    mg.add(&frame);
    let glass = mesh(plane_geo!(0.78, 1.28), basic_mat!({ color: 0x141922, transparent: true, opacity: 0.85 }));
    glass.position().z = 0.032;
    mg.add(&glass);
    // cracks
    let crack_mat = line_mat!({ color: 0xd8d0b8 });
    for _i in 0..4 {
        let mut pts = vec![];
        let mut x = rand2(-0.3, 0.3);
        let mut y = rand2(-0.5, 0.5);
        pts.push(v3(x, y, 0.033));
        for _s in 0..5 {
            x += rand2(-0.15, 0.15);
            y += rand2(-0.2, 0.2);
            pts.push(v3(x, y, 0.033));
        }
        let geo = Geo::from_points(&pts);
        mg.add(&line(geo, &crack_mat));
    }
    Built::new(mg, 1.4)
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

/// A stack of old books, hardbacks and paperbacks, laid askew
pub fn build_book_stack(g: &mut Game, cx: &mut PropsCtx) -> Built {
    let _ = (g, cx);
    // each book modelled in the batch `r02` at its real size (cloth
    // hardbacks of three sizes, their boards over the page block, their
    // spines rounded; a trade paperback), its covers, spine and pages
    // painted in one atlas for the stack (docs/PROPS.md)
    use crate::assets::props::prop_part_as_modelled;
    const CLOTHS: [usize; 8] = [0, 1, 2, 3, 4, 5, 6, 7];
    // (boards, spine, pages, thickness, board width): hardbacks A, B, C, the paperback
    const KINDS: [(&str, &str, &str, f64, f64); 4] = [
        ("Real__bookHbABoards", "Real__bookHbASpine", "Real__bookHbAPages", 0.0352, 0.146),
        ("Real__bookHbBBoards", "Real__bookHbBSpine", "Real__bookHbBPages", 0.0492, 0.164),
        ("Real__bookHbCBoards", "Real__bookHbCSpine", "Real__bookHbCPages", 0.0262, 0.128),
        ("Real__bookPbCovers", "Real__bookPbSpine", "Real__bookPbPages", 0.0234, 0.140),
    ];
    // the old draws, call for call: how many, each book's size, cloth,
    // place, turn and ribbon (now a slip of paper kept in it), then the one
    // that stood on top (now one more laid there, turned across the rest)
    let mut books: Vec<(usize, usize, f64, f64, f64, f64, Option<f64>)> = vec![];
    let n = irand(3, 6);
    for _ in 0..n {
        let w = rand2(0.18, 0.26);
        let h = rand2(0.035, 0.055);
        let _ = rand2(0.14, 0.2); // (its depth: the kind says it now)
        let cloth = choice(&CLOTHS);
        let px = rand2(-0.03, 0.03);
        let pz = rand2(-0.03, 0.03);
        let yaw = rand2(-0.3, 0.3);
        let slip = if rnd() < 0.25 { Some(rand2(-0.05, 0.05)) } else { None };
        // which book its size made it: a small hardback, a paperback, a
        // novel, a big one; its thickness scales it a little
        let u = (w - 0.18) / 0.08;
        let kind = if u < 0.22 {
            2
        } else if u < 0.47 {
            3
        } else if u < 0.78 {
            0
        } else {
            1
        };
        books.push((kind, cloth, 0.94 + (h - 0.035) * 6.0, px, pz, yaw, slip));
    }
    if rnd() < 0.5 {
        let cloth = choice(&CLOTHS);
        let px = rand2(-0.06, 0.06);
        let pz = rand2(-0.06, 0.06);
        let lean = rand2(-0.5, 0.5);
        let turn = rand2(0.0, PI * 2.0);
        books.push((if lean < 0.0 { 3 } else { 0 }, cloth, 1.0, px, pz, turn, None));
    }
    let atlas = book_atlas(books.iter().map(|b| (b.0 == 3, b.1)).collect());
    // one cell of the atlas (pixels, from its top left), inset a pixel
    let cell = |x0: f64, y0: f64, w: f64, h: f64| {
        let t = atlas.duplicate();
        t.set_offset((x0 + 1.0) / 512.0, 1.0 - (y0 + h - 1.0) / 512.0);
        t.set_repeat((w - 2.0) / 512.0, (h - 2.0) / 512.0);
        lambert!({ map: &t })
    };
    let pages = cell(0.0, 384.0, 512.0, 128.0);
    let paper = lambert!({ color: 0xcdc4ad });
    let bg = group();
    let mut y = 0.0;
    for (j, &(kind, _, s, px, pz, yaw, slip)) in books.iter().enumerate() {
        let (boards, spine, block, thick, width) = KINDS[kind];
        let book = group();
        book.position().set(px, y, pz);
        book.rotation().y = yaw;
        book.scale().set(s, s, s);
        let cover = cell((j % 4) as f64 * 128.0, (j / 4) as f64 * 128.0, 128.0, 128.0);
        let back = cell((j % 2) as f64 * 256.0, 256.0 + (j / 2) as f64 * 32.0, 256.0, 32.0);
        for (name, mat) in [(boards, &cover), (spine, &back), (block, &pages)] {
            if let Some(geo) = prop_part_as_modelled(name) {
                book.add(&mesh(geo, mat));
            }
        }
        if let (Some(x), Some(geo)) = (slip, prop_part_as_modelled("Real__bookSlip")) {
            let m = mesh(geo, &paper);
            m.position().set(x, thick * 0.5, -width / 2.0 + 0.04);
            book.add(&m);
        }
        bg.add(&book);
        y += thick * s;
    }
    Built::new(bg, y)
}

/// Book cloth, faded (sRGB): maroon, bottle green, navy, brown, tan
/// buckram, slate, olive, black gone brown
const BOOK_CLOTH: [(f64, f64, f64); 8] = [
    (98.0, 46.0, 42.0),
    (50.0, 76.0, 60.0),
    (46.0, 56.0, 78.0),
    (94.0, 68.0, 50.0),
    (152.0, 130.0, 98.0),
    (88.0, 100.0, 108.0),
    (98.0, 94.0, 62.0),
    (48.0, 44.0, 40.0),
];
/// A paperback's printed cover: its ground and its band
const BOOK_PRINT: [((f64, f64, f64), (f64, f64, f64)); 8] = [
    ((172.0, 118.0, 76.0), (204.0, 194.0, 170.0)),
    ((86.0, 106.0, 88.0), (202.0, 192.0, 168.0)),
    ((80.0, 96.0, 118.0), (200.0, 190.0, 168.0)),
    ((198.0, 190.0, 168.0), (128.0, 64.0, 54.0)),
    ((112.0, 60.0, 54.0), (198.0, 186.0, 160.0)),
    ((72.0, 98.0, 100.0), (200.0, 190.0, 168.0)),
    ((158.0, 136.0, 82.0), (52.0, 48.0, 44.0)),
    ((118.0, 116.0, 110.0), (196.0, 188.0, 170.0)),
];
const BOOK_TITLES: [&str; 20] = [
    "THE SILENT HOUSE",
    "COLLECTED POEMS",
    "A WINTER JOURNEY",
    "HOUSEHOLD MANAGEMENT",
    "ELEMENTS OF GEOMETRY",
    "THE NORTHERN SHORE",
    "LETTERS",
    "TALES OF THE SEA",
    "PRACTICAL ARITHMETIC",
    "THE OLD ROAD",
    "BIRDS OF THE FIELD",
    "GENERAL CHEMISTRY",
    "THE GARDEN YEAR",
    "SHORT STORIES",
    "THE HOLLOW TREE",
    "ROADS AND BRIDGES",
    "A SHORT HISTORY",
    "THE LONG AFTERNOON",
    "SELECTED ESSAYS",
    "THE MAP ROOM",
];
const BOOK_AUTHORS: [&str; 10] =
    ["H. MARLOWE", "E. M. GRAY", "J. ASHBY", "R. T. HALE", "M. WREN", "A. C. FORD", "L. DUNMORE", "S. KEMP", "F. ORME", "D. LATHAM"];

fn rgb(c: (f64, f64, f64), a: f64) -> String {
    format!("rgba({},{},{},{a})", c.0 as u8, c.1 as u8, c.2 as u8)
}

/// A stack's books painted in one 512 px atlas: each book's front cover
/// (128 px cells, two rows; its head to the left, its fore-edge up, as its
/// board's top takes it), its spine (256 x 32 cells, four rows; head to the
/// left, the front board up) and the page edges all of them share (the
/// bottom 128 px)
fn book_atlas(looks: Vec<(bool, usize)>) -> Tex {
    canvas_tex_opt(512, 512, false, move |ctx, _, _| {
        for (i, &(paperback, c)) in looks.iter().enumerate() {
            let title = BOOK_TITLES[((rnd() * 20.0) as usize).min(19)];
            let author = BOOK_AUTHORS[((rnd() * 10.0) as usize).min(9)];
            let (x0, y0) = ((i % 4) as f64 * 128.0, (i / 4) as f64 * 128.0);
            let (sx, sy) = ((i % 2) as f64 * 256.0, 256.0 + (i / 2) as f64 * 32.0);
            ctx.save();
            ctx.begin_path();
            ctx.rect(x0, y0, 128, 128);
            ctx.clip();
            if paperback {
                book_print_cover(ctx, x0, y0, c, title, author);
            } else {
                book_cloth_cover(ctx, x0, y0, c, title);
            }
            ctx.restore();
            ctx.save();
            ctx.begin_path();
            ctx.rect(sx, sy, 256, 32);
            ctx.clip();
            book_spine(ctx, sx, sy, paperback, c, title, author);
            ctx.restore();
        }
        book_pages(ctx, 0.0, 384.0, 512.0, 128.0);
    })
}

/// a cloth's weave: a fine speckle lighter and darker than its ground
fn book_weave(ctx: &mut Canvas, x0: f64, y0: f64, w: f64, h: f64, n: usize) {
    for k in 0..n {
        let (x, y) = (x0 + rnd() * w, y0 + rnd() * h);
        ctx.set_fill_style(if k % 2 == 0 { "rgba(255,245,225,0.10)" } else { "rgba(0,0,0,0.12)" });
        ctx.fill_rect(x, y, 1.0 + rnd(), 1);
    }
}

/// a soft blotch (a water mark, a ring, grime)
fn book_stain(ctx: &mut Canvas, x: f64, y: f64, r: f64, ring: bool, col: &str) {
    let gr = ctx.create_radial_gradient(x, y, r * if ring { 0.75 } else { 0.0 }, x, y, r);
    gr.add_color_stop(0, "rgba(0,0,0,0)");
    gr.add_color_stop(if ring { 0.85 } else { 0.0 }, col);
    gr.add_color_stop(1, "rgba(0,0,0,0)");
    ctx.set_fill_style(gr);
    ctx.fill_rect(x - r, y - r, 2.0 * r, 2.0 * r);
}

/// A cloth board: the ground faded where the light found it, the weave, a
/// blind-stamped rule, sometimes a title stamped in gilt, the corners
/// bumped through to the board, the edges rubbed, a stain
fn book_cloth_cover(ctx: &mut Canvas, x0: f64, y0: f64, c: usize, title: &str) {
    let col = BOOK_CLOTH[c];
    ctx.set_fill_style(rgb(col, 1.0));
    ctx.fill_rect(x0, y0, 128, 128);
    // faded toward the fore-edge, where it lay in the light
    let gr = ctx.create_linear_gradient(x0, y0 + 128.0, x0, y0);
    gr.add_color_stop(0, "rgba(210,200,180,0.0)");
    gr.add_color_stop(1, &format!("rgba(210,200,180,{})", 0.10 + 0.15 * rnd()));
    ctx.set_fill_style(gr);
    ctx.fill_rect(x0, y0, 128, 128);
    book_weave(ctx, x0, y0, 128.0, 128.0, 1800);
    // where another book lay on it the cloth kept its colour
    if rnd() < 0.5 {
        ctx.set_fill_style("rgba(0,0,0,0.10)");
        ctx.fill_rect(x0 + 128.0 * rnd() * 0.4, y0 + 40.0 + 60.0 * rnd(), 128, 128);
    }
    ctx.set_stroke_style("rgba(0,0,0,0.22)");
    ctx.set_line_width(1.3);
    ctx.stroke_rect(x0 + 7.0, y0 + 7.0, 114, 114);
    if rnd() < 0.45 {
        // the title stamped on the front, in the cover's own frame (its top at the head)
        ctx.save();
        ctx.translate(x0, y0 + 128.0);
        ctx.rotate(-PI / 2.0);
        ctx.scale(128.0 / 146.0, 128.0 / 222.0);
        ctx.set_fill_style("rgba(170,146,92,0.75)");
        ctx.set_text_align("center");
        ctx.set_font("bold 10px Times New Roman");
        ctx.fill_text_max(title, 73, 52, 100);
        ctx.restore();
    }
    // bumped corners (the fore-edge's, top left and right of the cell) and rubbed edges
    for (x, y) in [(x0, y0), (x0 + 128.0, y0)] {
        book_stain(ctx, x, y, 7.0 + rnd() * 6.0, false, "rgba(176,164,140,0.75)");
    }
    for (x, y, w, h) in [(x0, y0, 128.0, 3.0), (x0, y0, 3.0, 128.0), (x0 + 125.0, y0, 3.0, 128.0)] {
        ctx.set_fill_style("rgba(200,190,170,0.22)");
        ctx.fill_rect(x, y, w, h);
    }
    if rnd() < 0.6 {
        let (x, y) = (x0 + 20.0 + rnd() * 88.0, y0 + 20.0 + rnd() * 88.0);
        book_stain(ctx, x, y, 10.0 + rnd() * 18.0, rnd() < 0.5, "rgba(40,30,20,0.22)");
    }
}

/// A paperback's front: its printed ground and band, the title, the
/// author, the publisher's mark; creased, its corners and edges worn white
fn book_print_cover(ctx: &mut Canvas, x0: f64, y0: f64, c: usize, title: &str, author: &str) {
    let (ground, band) = BOOK_PRINT[c];
    ctx.save();
    ctx.translate(x0, y0 + 128.0);
    ctx.rotate(-PI / 2.0);
    ctx.scale(128.0 / 140.0, 128.0 / 216.0);
    // the cover, 140 x 216 (mm): its spine at the left
    ctx.set_fill_style(rgb(ground, 1.0));
    ctx.fill_rect(0, 0, 140, 216);
    ctx.set_fill_style(rgb(band, 1.0));
    ctx.fill_rect(0, 70, 140, 70);
    ctx.set_fill_style(rgb(ground, 0.9));
    ctx.set_text_align("center");
    ctx.set_font("bold 13px Gill Sans");
    for (k, line) in title.splitn(2, ' ').enumerate() {
        ctx.fill_text_max(line, 70, 98.0 + k as f64 * 17.0, 120);
    }
    ctx.set_fill_style(rgb(band, 0.9));
    ctx.set_font("10px Gill Sans");
    ctx.fill_text_max(author, 70, 40, 120);
    ctx.begin_path();
    ctx.arc(70, 185, 9, 0, 7);
    ctx.fill();
    // a crease where it was bent back at the spine, and across a corner
    ctx.set_stroke_style("rgba(235,228,212,0.7)");
    ctx.set_line_width(1);
    ctx.begin_path();
    ctx.move_to(9, 0);
    ctx.line_to(10, 216);
    ctx.stroke();
    ctx.begin_path();
    ctx.move_to(140.0 - 30.0 * rnd() - 10.0, 216);
    ctx.line_to(140, 216.0 - 30.0 * rnd() - 12.0);
    ctx.stroke();
    // the edges worn white
    ctx.set_fill_style("rgba(235,228,212,0.35)");
    for (x, y, w, h) in [(0.0, 0.0, 140.0, 3.0), (0.0, 213.0, 140.0, 3.0), (137.0, 0.0, 3.0, 216.0)] {
        ctx.fill_rect(x, y, w, h);
    }
    ctx.restore();
    for (x, y) in [(x0, y0), (x0 + 128.0, y0)] {
        book_stain(ctx, x, y, 6.0 + rnd() * 5.0, false, "rgba(236,230,214,0.7)");
    }
    book_weave(ctx, x0, y0, 128.0, 128.0, 500);
    if rnd() < 0.6 {
        let (x, y) = (x0 + 20.0 + rnd() * 88.0, y0 + 20.0 + rnd() * 88.0);
        book_stain(ctx, x, y, 12.0 + rnd() * 16.0, rnd() < 0.6, "rgba(60,44,24,0.25)");
    }
}

/// A spine (head at the left, the front board up): cloth faded more than
/// its boards, the title and author stamped (gilt, or ink on a light
/// cloth), the rules at head and tail, the ends frayed; or a paperback's
/// printed spine, cracked along its length where it was read
fn book_spine(ctx: &mut Canvas, x0: f64, y0: f64, paperback: bool, c: usize, title: &str, author: &str) {
    let (ink, ground) = if paperback {
        let (g0, b0) = BOOK_PRINT[c];
        ctx.set_fill_style(rgb(g0, 1.0));
        ctx.fill_rect(x0, y0, 256, 32);
        ctx.set_fill_style(rgb(b0, 1.0));
        ctx.fill_rect(x0 + 70.0, y0, 110, 32);
        (rgb(g0, 0.95), b0)
    } else {
        let col = BOOK_CLOTH[c];
        let f = 0.18 + 0.2 * rnd();
        let faded = (col.0 + (196.0 - col.0) * f, col.1 + (186.0 - col.1) * f, col.2 + (166.0 - col.2) * f);
        ctx.set_fill_style(rgb(faded, 1.0));
        ctx.fill_rect(x0, y0, 256, 32);
        book_weave(ctx, x0, y0, 256.0, 32.0, 900);
        let light = col.0 + col.1 + col.2 > 300.0;
        ctx.set_fill_style(if light { "rgba(40,30,24,0.8)" } else { "rgba(180,152,92,0.8)" });
        for x in [x0 + 12.0, x0 + 15.0, x0 + 241.0, x0 + 244.0] {
            ctx.fill_rect(x, y0, 1.2, 32);
        }
        ((if light { "rgba(40,30,24,0.85)" } else { "rgba(184,156,96,0.85)" }).to_string(), faded)
    };
    ctx.set_fill_style(ink.as_str());
    ctx.set_text_align("left");
    ctx.set_text_baseline("middle");
    ctx.set_font(if paperback { "bold 9px Gill Sans" } else { "bold 9px Times New Roman" });
    ctx.fill_text_max(title, x0 + if paperback { 78.0 } else { 30.0 }, y0 + 17.0, if paperback { 96.0 } else { 140.0 });
    ctx.set_font(if paperback { "7px Gill Sans" } else { "7px Times New Roman" });
    if paperback {
        ctx.set_fill_style(rgb(ground, 0.9));
        ctx.fill_text_max(author, x0 + 186.0, y0 + 17.0, 64);
    } else {
        ctx.fill_text_max(author, x0 + 182.0, y0 + 17.0, 54);
    }
    ctx.set_text_baseline("alphabetic");
    // the stamping rubbed: specks of the ground over it
    for _ in 0..140 {
        ctx.set_fill_style(rgb(ground, 0.6));
        ctx.fill_rect(x0 + rnd() * 256.0, y0 + rnd() * 32.0, 1.5, 1.5);
    }
    if paperback {
        ctx.set_stroke_style("rgba(236,230,214,0.75)");
        ctx.set_line_width(0.8);
        for _ in 0..(2.0 + rnd() * 4.0) as usize {
            let y = y0 + 4.0 + rnd() * 24.0;
            let x = x0 + rnd() * 60.0;
            ctx.begin_path();
            ctx.move_to(x, y);
            ctx.line_to(x + 60.0 + rnd() * 140.0, y + rnd() * 2.0 - 1.0);
            ctx.stroke();
        }
    } else {
        // frayed at head and tail
        for x in [x0, x0 + 250.0] {
            ctx.set_fill_style("rgba(200,188,166,0.35)");
            ctx.fill_rect(x, y0, 6, 32);
        }
    }
    // dust settled on its upper edge
    let gr = ctx.create_linear_gradient(x0, y0, x0, y0 + 8.0);
    gr.add_color_stop(0, "rgba(150,140,120,0.35)");
    gr.add_color_stop(1, "rgba(150,140,120,0)");
    ctx.set_fill_style(gr);
    ctx.fill_rect(x0, y0, 256, 8);
}

/// Page edges: old paper gone yellow, the leaves' fine lines, tanned
/// darker toward the boards, foxed, grimy
fn book_pages(ctx: &mut Canvas, x0: f64, y0: f64, w: f64, h: f64) {
    ctx.set_fill_style("#d2c4a0");
    ctx.fill_rect(x0, y0, w, h);
    let mut y = y0;
    while y < y0 + h {
        ctx.set_fill_style(format!("rgba(90,70,40,{})", 0.05 + 0.12 * rnd()));
        ctx.fill_rect(x0, y, w, 0.6 + rnd() * 0.8);
        y += 1.0 + rnd() * 2.0;
    }
    for (ya, yb) in [(y0, y0 + 16.0), (y0 + h, y0 + h - 16.0)] {
        let gr = ctx.create_linear_gradient(x0, ya, x0, yb);
        gr.add_color_stop(0, "rgba(120,86,40,0.45)");
        gr.add_color_stop(1, "rgba(120,86,40,0)");
        ctx.set_fill_style(gr);
        ctx.fill_rect(x0, y0, w, h);
    }
    for _ in 0..60 {
        let (x, y) = (x0 + rnd() * w, y0 + rnd() * h);
        book_stain(ctx, x, y, 1.0 + rnd() * 3.0, false, "rgba(130,84,40,0.35)");
    }
    let gr = ctx.create_linear_gradient(x0, y0, x0 + w, y0);
    gr.add_color_stop(0, "rgba(60,50,40,0.15)");
    gr.add_color_stop(0.5, "rgba(60,50,40,0)");
    gr.add_color_stop(1, "rgba(60,50,40,0.15)");
    ctx.set_fill_style(gr);
    ctx.fill_rect(x0, y0, w, h);
}

/// A bentwood coat stand, most often with an overcoat hung on it by its loop
pub fn build_coat_rack(g: &mut Game, cx: &mut PropsCtx) -> Built {
    let _ = (g, cx);
    // a Thonet-style stand (74 in: a turned pole, four bent legs and their
    // ring, six coat hooks and six hat hooks) and a wool overcoat hanging by
    // the loop in its collar from the front coat hook, both modelled in the
    // batch `r02` (docs/PROPS.md)
    use crate::assets::props::prop_part_as_modelled;
    let rg = group();
    // dark-stained beech, its varnish worn
    let wood = lambert!({ color: 0x3f2819 });
    for name in ["Real__coatStandPole", "Real__coatStandLegs", "Real__coatStandHooks"] {
        if let Some(geo) = prop_part_as_modelled(name) {
            rg.add(&mesh(geo, &wood));
        }
    }
    // the old draw: a coat on seven stands of ten; what it drew says which
    let r = rnd();
    if r < 0.7 {
        // charcoal, navy, camel, brown, loden: wool, dulled
        let colours = [0x47474au32, 0x343a48, 0x86694a, 0x56463a, 0x4a5444];
        let wool = lambert!({ color: colours[((r / 0.7 * 5.0) as usize).min(4)] });
        let horn = lambert!({ color: 0x2b211a });
        let coat = group();
        // the loop rests in the bottom of the hook's J
        coat.position().set(0, 1.515, 0.128);
        for (name, mat) in [("Real__coat", &wool), ("Real__coatButtons", &horn)] {
            if let Some(geo) = prop_part_as_modelled(name) {
                coat.add(&mesh(geo, mat));
            }
        }
        rg.add(&coat);
    }
    Built::new(rg, 1.88)
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

/// A heap of debris swept up in a store or a tunnel: grit and plaster dust,
/// bits of brick and concrete, and on it broken pieces of what the place
/// was built of
pub fn build_pile(g: &mut Game, cx: &mut PropsCtx) -> Built {
    let _ = (g, cx);
    // the mound and the pieces are modelled in the batch `r02`; the pieces
    // lie on the mound, whose height is PL_H (1 - (x/RX)^2 - (z/RZ)^2)^1.3
    // (docs/PROPS.md)
    use crate::assets::props::prop_part_as_modelled;
    const RX: f64 = 0.28;
    const RZ: f64 = 0.235;
    const PL_H: f64 = 0.14;
    let heap_h = |x: f64, z: f64| {
        let q = 1.0 - (x / RX).powi(2) - (z / RZ).powi(2);
        if q > 0.0 {
            PL_H * q.powf(1.3)
        } else {
            0.0
        }
    };
    let pg_ = group();
    let dust = lambert!({ color: 0x858078 });
    let concrete = lambert!({ color: 0x64615b });
    let plaster = lambert!({ color: 0xa6a194 });
    let brick = lambert!({ color: 0x634438 });
    for (name, mat) in [
        ("Real__pileHeap", &dust),
        ("Real__pileConcreteBits", &concrete),
        ("Real__pilePlasterBits", &plaster),
        ("Real__pileBrickBits", &brick),
    ] {
        if let Some(geo) = prop_part_as_modelled(name) {
            pg_.add(&mesh(geo, mat));
        }
    }
    // the dust it has shed round it
    let shed = mesh(plane_geo!(1.2, 1.0), pile_dust_mat());
    shed.rotation().x = -PI / 2.0;
    shed.position().y = 0.0015;
    pg_.add(&shed);
    // the old draws, call for call (the bound `irand(4, 8)` redrawn every
    // pass): a flat piece or a lump, its size and colour, where, how turned
    let mut pieces = vec![];
    let mut i = 0i64;
    while i < irand(4, 8) {
        let piece = if choice(&[0, 1]) == 0 {
            let (a, _, _) = (rand2(0.1, 0.3), rand2(0.05, 0.2), rand2(0.1, 0.3));
            let n = irand(0x111111, 0xaaaaaa);
            let u = (n - 0x111111) as f64 / (0xaaaaaa - 0x111111) as f64;
            // drywall, a ceiling tile's corner, a brick
            let kind = if u < 0.38 {
                0
            } else if u < 0.72 {
                1
            } else {
                2
            };
            (kind, 0.92 + (a - 0.1) * 0.8)
        } else {
            let r = rand2(0.06, 0.12);
            let n = irand(0x222222, 0xbb9955);
            let u = (n - 0x222222) as f64 / (0xbb9955 - 0x222222) as f64;
            // a lump of concrete, half a brick, a small lump
            let kind = if u < 0.4 {
                3
            } else if u < 0.7 {
                4
            } else {
                5
            };
            (kind, 0.92 + (r - 0.06) / 0.06 * 0.16)
        };
        let (x, _, z) = (rand2(-0.3, 0.3), rand2(0.05, 0.35), rand2(-0.3, 0.3));
        let (a, b, c) = (rand2(0.0, 7.0), rand2(0.0, 7.0), rand2(0.0, 7.0));
        pieces.push((piece.0, piece.1, x, z, (a - 3.5) * 0.035, b, (c - 3.5) * 0.035));
        i += 1;
    }
    // each laid on the mound, or on what was laid before it there, tipped to
    // the slope
    const KINDS: [(&str, u32, f64, f64); 6] = [
        // part, colour, footprint radius, thickness
        ("Real__pileDrywall", 0xb3ad9e, 0.17, 0.013),
        ("Real__pileTile", 0xaea690, 0.15, 0.016),
        ("Real__pileBrick", 0x664638, 0.10, 0.057),
        ("Real__pileConcrete", 0x7a776f, 0.09, 0.08),
        ("Real__pileHalfBrick", 0x624234, 0.06, 0.057),
        ("Real__pileConcreteSmall", 0x7d7a72, 0.055, 0.05),
    ];
    let mut laid: Vec<(f64, f64, f64, f64)> = vec![]; // x, z, radius, top
    for (kind, s, x, z, tx, yaw, tz) in pieces {
        let (name, colour, rad, thick) = KINDS[kind];
        let mut y = heap_h(x, z) - 0.006;
        for &(lx, lz, lr, top) in &laid {
            let d = ((x - lx).powi(2) + (z - lz).powi(2)).sqrt();
            if d < (rad + lr) * 0.75 {
                y = y.max(top - 0.01 - 0.02 * d / (rad + lr));
            }
        }
        laid.push((x, z, rad * s, y + thick * s));
        let e = 0.01;
        let sx = (heap_h(x + e, z) - heap_h(x - e, z)) / (2.0 * e);
        let sz = (heap_h(x, z + e) - heap_h(x, z - e)) / (2.0 * e);
        let Some(geo) = prop_part_as_modelled(name) else { continue };
        let tilt = group();
        tilt.position().set(x, y, z);
        // a broad flat piece bridges the lumps: it takes less of the slope
        let k = if kind < 2 { 0.55 } else { 0.85 };
        tilt.rotation().set(-(sz * k).atan() + tx, 0, (sx * k).atan() + tz);
        let mat = if kind == 1 { lambert!({ map: &ceiling_tile_tex() }) } else { lambert!({ color: colour }) };
        let m = mesh(geo, mat);
        m.scale().set(s, s, s);
        m.rotation().y = yaw;
        tilt.add(&m);
        // nothing of it through the floor
        tilt.update_matrix_world(true);
        let mut b = Box3::default();
        b.set_from_object(&tilt);
        if !b.is_empty() && b.min.y < 0.0 {
            tilt.position().y = y - b.min.y;
        }
        pg_.add(&tilt);
    }
    Built::new(pg_, 0.4)
}

/// Pale dust on the floor round a heap of rubble: thick at the heap, thin
/// and patchy away from it, grit strewn in it, a few drags of a broom
fn pile_dust_mat() -> Mat {
    let tex = canvas_tex_opt(256, 256, false, |ctx, w, h| {
        ctx.clear_rect(0, 0, w, h);
        for _ in 0..26 {
            let a = rnd() * 6.3;
            let d = rnd() * 0.2;
            let (x, y) = (w * (0.5 + a.cos() * d), h * (0.5 + a.sin() * d));
            let r = w * (0.1 + 0.16 * rnd());
            let (squash, turn) = (0.5 + 0.5 * rnd(), rnd() * 3.0);
            // squashed as a whole, so the fade reaches its edge (an ellipse
            // cut from a round fade leaves a hard rim)
            ctx.save();
            ctx.translate(x, y);
            ctx.rotate(turn);
            ctx.scale(1, squash);
            let gr = ctx.create_radial_gradient(0, 0, 0, 0, 0, r);
            gr.add_color_stop(0, "rgba(140,135,124,0.4)");
            gr.add_color_stop(1, "rgba(140,135,124,0)");
            ctx.set_fill_style(gr);
            ctx.begin_path();
            ctx.arc(0, 0, r, 0, 7);
            ctx.fill();
            ctx.restore();
        }
        for k in 0..500 {
            let a = rnd() * 6.3;
            let d = 0.15 + 0.3 * rnd() * rnd();
            let (x, y) = (w * (0.5 + a.cos() * d), h * (0.5 + a.sin() * d));
            ctx.set_fill_style(if k % 3 == 0 { "rgba(70,66,60,0.7)" } else { "rgba(170,164,152,0.6)" });
            let s = 0.8 + rnd() * 2.0;
            ctx.fill_rect(x, y, s, s * (0.5 + rnd()));
        }
        ctx.set_stroke_style("rgba(160,154,142,0.14)");
        for _ in 0..7 {
            let a = rnd() * 6.3;
            let (x, y) = (w * (0.5 + a.cos() * 0.2), h * (0.5 + a.sin() * 0.2));
            ctx.set_line_width(2.0 + rnd() * 4.0);
            ctx.begin_path();
            ctx.move_to(x, y);
            ctx.line_to(x + a.cos() * 50.0, y + a.sin() * 50.0);
            ctx.stroke();
        }
    });
    lambert!({ map: &tex, transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 })
}

/// A mineral fibre ceiling tile's face: off-white gone yellow, its fine
/// fissures, a water stain
fn ceiling_tile_tex() -> Tex {
    canvas_tex_opt(128, 128, false, |ctx, w, h| {
        ctx.set_fill_style("#b4ab94");
        ctx.fill_rect(0, 0, w, h);
        ctx.set_stroke_style("rgba(70,62,50,0.55)");
        ctx.set_line_cap("round");
        for _ in 0..140 {
            let (x, y) = (rnd() * w, rnd() * h);
            let a = rnd() * 6.3;
            let l = 2.0 + rnd() * 5.0;
            ctx.set_line_width(0.6 + rnd() * 0.8);
            ctx.begin_path();
            ctx.move_to(x, y);
            ctx.quadratic_curve_to(x + (a + 1.0).cos() * l, y + (a + 1.0).sin() * l, x + a.cos() * l * 1.6, y + a.sin() * l * 1.6);
            ctx.stroke();
        }
        let (x, y, r) = (w * (0.3 + 0.4 * rnd()), h * (0.3 + 0.4 * rnd()), w * 0.3);
        let gr = ctx.create_radial_gradient(x, y, r * 0.6, x, y, r);
        gr.add_color_stop(0, "rgba(120,96,60,0.12)");
        gr.add_color_stop(0.85, "rgba(110,84,50,0.35)");
        gr.add_color_stop(1, "rgba(110,84,50,0)");
        ctx.set_fill_style(gr);
        ctx.fill_rect(0, 0, w, h);
    })
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

/// Large floor plant (fiddle-leaf-fig style)
pub fn build_floor_plant(g: &mut Game, cx: &mut PropsCtx) -> Built {
    let _ = (g, cx);
    let fp = group();
    // pot with drainage rim & fake crack
    let pot_mat = lambert!({ color: 0x6a4028 });
    let rim_mat = lambert!({ color: 0x7a4a30 });
    let pot = mesh(pg("Props__floorPlant_pot", cyl_geo!(0.25, 0.19, 0.4, 16)), &pot_mat);
    pot.position().y = 0.2;
    fp.add(&pot);
    let rim = mesh(pg("Props__floorPlant_rim", cyl_geo!(0.27, 0.26, 0.03, 16)), &rim_mat);
    rim.position().y = 0.40;
    fp.add(&rim);
    // crack line
    let crack = mesh(box_geo!(0.005, 0.28, 0.01), lambert!({ color: 0x1a1008 }));
    // on the tapered wall (r 0.223 at y 0.22), leaning out with it; the diagonal
    // is a turn about the radial axis so it stays in the surface (was 2.7 cm off)
    crack.position().set(0.2245, 0.22, 0);
    crack.rotation().set(0.15, 0, -0.149);
    fp.add(&crack);
    // soil + moss on top
    let soil = mesh(pg("Props__floorPlant_soil", cyl_geo!(0.23, 0.23, 0.02, 14)), lambert!({ color: 0x2a1a10 }));
    soil.position().y = 0.41;
    fp.add(&soil);
    // moss patches on soil
    for _m in 0..4 {
        let moss = mesh(circle_geo!(rand2(0.04, 0.08), 8), lambert!({ color: 0x4a7a35, emissive: 0x0a1608 }));
        moss.rotation().x = -PI / 2.0;
        let x = rand2(-0.15, 0.15);
        let z = rand2(-0.15, 0.15);
        moss.position().set(x, 0.42, z);
        fp.add(&moss);
    }
    // gnarly branching trunk — main + 2 side branches
    let trunk_mat = lambert!({ color: 0x3a2a1a });
    let trunk = mesh(pg("Props__floorPlant_trunk", cyl_geo!(0.025, 0.04, 1.4, 8)), &trunk_mat);
    trunk.position().y = 1.1;
    fp.add(&trunk);
    let branch1 = mesh(pg("Props__floorPlant_branchA", cyl_geo!(0.014, 0.022, 0.5, 6)), &trunk_mat);
    branch1.position().set(0.15, 1.5, 0.05);
    branch1.rotation().z = -0.7;
    fp.add(&branch1);
    let branch2 = mesh(pg("Props__floorPlant_branchB", cyl_geo!(0.014, 0.02, 0.4, 6)), &trunk_mat);
    branch2.position().set(-0.12, 1.3, -0.05);
    branch2.rotation().z = 0.6;
    fp.add(&branch2);
    // broad leaves with curl — placed along trunk and branches
    let mut leaf_positions: Vec<(f64, f64, f64)> = vec![];
    for i in 0..4 {
        leaf_positions.push((0.0, 0.7 + i as f64 * 0.22, 0.0));
    } // base: 'trunk'
    leaf_positions.push((0.25, 1.7, 0.1)); // base: 'branch1'
    leaf_positions.push((0.28, 1.85, -0.05)); // base: 'branch1'
    leaf_positions.push((-0.20, 1.5, -0.08)); // base: 'branch2'
    leaf_positions.push((-0.24, 1.65, 0.05)); // base: 'branch2'
    let big_leaf_colors = [0x2a5a28u32, 0x357a30, 0x3a6a30, 0x225022];
    for &(lx, ly, lz) in &leaf_positions {
        let (len, wid, color, curl) = (rand2(0.22, 0.32), rand2(0.10, 0.14), choice(&big_leaf_colors), rand2(0.18, 0.32));
        let leaf = make_leaf_shape(len, wid, color, curl);
        let x = lx + rand2(-0.05, 0.05);
        let z = lz + rand2(-0.05, 0.05);
        leaf.position().set(x, ly, z);
        let (a, b, c) = (rand2(-0.4, -0.1), rand2(0.0, PI * 2.0), rand2(-0.3, 0.3));
        leaf.rotation().set(a, b, c);
        fp.add(&leaf);
        // leaf vein down center
        let vein = mesh(box_geo!(0.003, 0.22, 0.002), lambert!({ color: 0x1a3a1a }));
        leaf.add(&vein);
        vein.position().set(0, 0.11, 0.005);
    }
    // 1-2 yellowing dying leaves
    for _i in 0..2 {
        let (len, wid, curl) = (rand2(0.16, 0.22), rand2(0.08, 0.11), rand2(0.25, 0.45));
        let dying = make_leaf_shape(len, wid, 0x8a7a2a, curl);
        let (x, y, z) = (rand2(-0.12, 0.12), rand2(0.6, 1.2), rand2(-0.12, 0.12));
        dying.position().set(x, y, z);
        let (a, b, c) = (rand2(-1.2, -0.8), rand2(0.0, PI * 2.0), rand2(-0.3, 0.3));
        dying.rotation().set(a, b, c);
        fp.add(&dying);
    }
    // dropped leaves on floor at base
    for _i in 0..4 {
        let dropped = mesh(
            pg("Props__plant_fallenLeaf", box_geo!(rand2(0.08, 0.14), 0.003, rand2(0.05, 0.08))),
            lambert!({ color: [0x5a4020u32, 0x7a6828, 0x604520][irand(0, 2) as usize] }),
        );
        let x = rand2(-0.35, 0.35);
        let z = rand2(-0.35, 0.35);
        dropped.position().set(x, 0.005, z);
        dropped.rotation().y = rand2(0.0, 7.0);
        dropped.rotation().x = rand2(-0.2, 0.2);
        fp.add(&dropped);
    }
    // cobweb between two branches (occasional)
    if rnd() < 0.4 {
        let web_mat = basic_mat!({ color: 0xe8e0d0, transparent: true, opacity: 0.35 });
        let web = mesh(plane_geo!(0.18, 0.14), &web_mat);
        web.position().set(0.05, 1.35, 0.06);
        let (a, b, c) = (rand2(-0.5, 0.5), rand2(0.0, 7.0), rand2(-0.5, 0.5));
        web.rotation().set(a, b, c);
        fp.add(&web);
    }
    Built::new(fp, 1.9)
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
