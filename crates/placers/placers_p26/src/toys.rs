//! Toys, children's items and restroom/luggage fixtures (R408-R421).
use crate::{buf_geo, open_tiles, pick, set_rot_order, wall_candidates};
use game::prelude::*;

// the carnival mask: batch `t09`'s (`toys/t09.rs`)
mod t09;
pub use t09::place_small_masks;
// the plush teddy and the small doll: batch `t14`'s (`toys/t14.rs`)
mod t14;
pub use t14::place_plush_bears;

/// R421 small pacifier: 8% chance per level. Nipple cylinder +
/// guard disc + ring handle.
pub fn place_small_pacifiers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let guard_colors = [0xd8a0c0, 0xa0c0d8, 0xd8d0a0, 0xf0f0f0];
    let guard_mat = lambert!({ color: pick(&guard_colors) });
    let nip_mat = lambert!({ color: 0xe8dcc0 });
    let paci = group();
    // Nipple: soft small cylinder.
    let nipple = mesh(pg("SmallPacifiers__nipple", cyl_geo!(0.008, 0.010, 0.020, 6)), &nip_mat);
    nipple.rotation().z = PI / 2.0;
    nipple.position().set(0.015, 0.014, 0);
    paci.add(&nipple);
    // Guard disc: shield behind nipple.
    let guard = mesh(pg("SmallPacifiers__shield", cyl_geo!(0.020, 0.020, 0.005, 10)), &guard_mat);
    guard.rotation().z = PI / 2.0;
    guard.position().set(0.005, 0.014, 0);
    paci.add(&guard);
    // Ring handle behind guard (half-torus).
    let ring = mesh(pg("SmallPacifiers__ring", torus_geo!(0.010, 0.002, 4, 10)), &guard_mat);
    ring.position().set(-0.008, 0.014, 0);
    paci.add(&ring);
    paci.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    paci.rotation().y = rnd() * PI * 2.0;
    g.world.add(&paci);
}

/// R420 small toy drum: 8% chance per level. Cylinder body + top +
/// bottom membrane discs + loose strap.
pub fn place_toy_drums(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let body_colors = [0xd82020, 0x2050d8, 0xd8b820, 0x30a030];
    let body_mat = lambert!({ color: pick(&body_colors) });
    let mem_mat = lambert!({ color: 0xd8ccb0 });
    let strap_mat = lambert!({ color: 0x4a3018 });
    let drum = group();
    // Body: vertical cylinder.
    let body = mesh(pg("ToyDrums__body", cyl_geo!(0.06, 0.06, 0.09, 12)), &body_mat);
    body.position().y = 0.045;
    drum.add(&body);
    // Top membrane disc.
    let top_m = mesh(circle_geo!(0.058, 12), &mem_mat);
    top_m.rotation().x = -PI / 2.0;
    top_m.position().y = 0.091;
    drum.add(&top_m);
    // Bottom membrane disc.
    let bot_m = mesh(circle_geo!(0.058, 12), &mem_mat);
    bot_m.rotation().x = PI / 2.0;
    bot_m.position().y = -0.001;
    drum.add(&bot_m);
    // Loose carry strap lying face-up on the floor, one end tucked under the
    // drum's rim at +X, trailing off 0.3 rad from the tangent.
    let strap = mesh(plane_geo!(0.015, 0.20), &strap_mat);
    strap.rotation().x = -PI / 2.0;
    strap.position().set(0.06 + 0.3f64.sin() * 0.095, 0.0015, 0.3f64.cos() * 0.095);
    strap.rotation().z = 0.3;
    drum.add(&strap);
    drum.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    drum.rotation().y = rnd() * PI * 2.0;
    g.world.add(&drum);
}

/// R420 small hat: 10% chance per level. Cylinder crown + flat brim.
pub fn place_small_hats(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let hat_colors = [0x181410, 0x4a3018, 0x2a2418, 0x604828, 0x681818];
    let hat_mat = lambert!({ color: pick(&hat_colors) });
    let band_mat = lambert!({ color: 0x2a2418 });
    let hat = group();
    // Brim: wider flat disc.
    let brim = mesh(pg("SmallHats__brim", cyl_geo!(0.10, 0.10, 0.008, 14)), &hat_mat);
    brim.position().y = 0.004;
    hat.add(&brim);
    // Crown: taller narrower cylinder.
    let crown = mesh(pg("SmallHats__crown", cyl_geo!(0.055, 0.055, 0.08, 12)), &hat_mat);
    crown.position().y = 0.048;
    hat.add(&crown);
    // Small band ring around crown base.
    let band = mesh(pg("SmallHats__band", torus_geo!(0.055, 0.005, 4, 12)), &band_mat);
    band.rotation().x = PI / 2.0;
    band.position().y = 0.014;
    hat.add(&band);
    hat.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    hat.rotation().y = rnd() * PI * 2.0;
    // 25% chance tipped on side (brim flat instead of vertical).
    if rnd() < 0.25 {
        hat.rotation().z = PI / 2.0;
        hat.position().y = 0.10;
    }
    g.world.add(&hat);
}

/// R419 small yo-yo: 10% chance per level. 2 flat colored discs +
/// connecting axle + short dangling string.
pub fn place_small_yo_yos(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let yoyo_colors = [0xd82020, 0x2050d8, 0xd8b820, 0x30a030, 0x8020d8];
    let yoyo_mat = lambert!({ color: pick(&yoyo_colors) });
    let axle_mat = lambert!({ color: 0x2a2418 });
    let string_mat = lambert!({ color: 0xe8e0d0 });
    let yoyo = group();
    // 2 flat discs (yo-yo halves).
    for x_off in [-0.010, 0.010] {
        let disc = mesh(pg("SmallYoYos__half", cyl_geo!(0.024, 0.024, 0.006, 12)), &yoyo_mat);
        disc.rotation().z = PI / 2.0;
        disc.position().set(x_off, 0.024, 0);
        yoyo.add(&disc);
    }
    // Central axle cylinder.
    let axle = mesh(pg("SmallYoYos__axle", cyl_geo!(0.005, 0.005, 0.020, 6)), &axle_mat);
    axle.rotation().z = PI / 2.0;
    axle.position().y = 0.024;
    yoyo.add(&axle);
    // Short dangling string: leaves the underside of the axle between the
    // halves and slopes down to the floor 10 cm out.
    let string = mesh(pg("SmallYoYos__string", cyl_geo!(0.001, 0.001, 0.10, 4)), &string_mat);
    string.position().set(0, 0.0105, 0.049);
    string.rotation().x = PI / 2.0 + 0.18;
    yoyo.add(&string);
    yoyo.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    yoyo.rotation().y = rnd() * PI * 2.0;
    g.world.add(&yoyo);
}

/// R418 small kite: 8% chance per level. Diamond-shape plane + 3
/// small bow ribbons on tail. Laid flat on floor.
pub fn place_small_kites(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let kite_colors = [0xd82020, 0x2050d8, 0xd8b820, 0x30a030, 0xd050a0];
    let k_mat = lambert!({ color: pick(&kite_colors), side: DoubleSide });
    let bow_mat = lambert!({ color: pick(&kite_colors) });
    let string_mat = lambert!({ color: 0x181410 });
    let kite = group();
    // Diamond kite: 2 triangles arranged in diamond (custom geometry).
    let k_geo = buf_geo(
        &[
            0.0, 0.0, 0.12, // top
            -0.09, 0.0, 0.0, // left
            0.0, 0.0, -0.14, // bottom (longer)
            0.09, 0.0, 0.0, // right
        ],
        &[0, 1, 2, 0, 2, 3],
    );
    let k_mesh = mesh(k_geo, &k_mat);
    k_mesh.position().y = 0.003;
    kite.add(&k_mesh);
    // Tail: thin cylinder extending from bottom of kite.
    let tail = mesh(pg("SmallKites__tail", cyl_geo!(0.002, 0.002, 0.25, 4)), &string_mat);
    tail.rotation().x = PI / 2.0;
    tail.position().set(0, 0.004, -0.24);
    kite.add(&tail);
    // 3 bow ribbons on tail.
    for b in 0..3 {
        let bow = mesh(pg("SmallKites__bow", plane_geo!(0.02, 0.015)), &bow_mat);
        bow.rotation().x = -PI / 2.0;
        bow.rotation().z = rnd() * PI * 2.0;
        bow.position().set(0, 0.005, -0.15 - b as f64 * 0.06);
        kite.add(&bow);
    }
    kite.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    kite.rotation().y = rnd() * PI * 2.0;
    g.world.add(&kite);
}

/// R418 small pinwheel: 10% chance per level. A paper party pinwheel
/// (Blender batch r17): a 10 cm square of printed paper cut and curled into
/// four vanes, pinned through a bead into a 28 cm birch dowel, dropped and
/// lying face up on the floor (never standing on its stick's point): the
/// dowel on the floor, the wheel over its end, the curls standing up, the
/// paper faded, water-marked and grimed. Three in ten stir on the pin in a
/// draught.
pub fn place_small_pinwheels(g: &mut Game, _dist: &[i32]) {
    use game::assets::props::prop_part_as_modelled;
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // the four vanes' papers: the old red, blue, yellow and green, faded
    let papers: [u32; 4] = [0xa4473a, 0x3e5a88, 0xbf9c44, 0x4b7449];
    let pinwheel = group();
    if let Some(geo) = prop_part_as_modelled("Real__pinwheelStick") {
        pinwheel.add(&mesh(geo, lambert!({ color: 0xffffff, map: &pinwheel_dowel_tex() })));
    }
    // the wheel turns on the pin about the hub (the parts' origin)
    let wheel = group();
    if let Some(geo) = prop_part_as_modelled("Real__pinwheelVane") {
        for (k, &paper) in papers.iter().enumerate() {
            let vane = mesh(geo.clone(), lambert!({ color: 0xffffff, map: &pinwheel_paper_tex(paper), side: DoubleSide }));
            vane.rotation().y = k as f64 * PI / 2.0;
            wheel.add(&vane);
        }
    }
    if let Some(geo) = prop_part_as_modelled("Real__pinwheelPin") {
        wheel.add(&mesh(geo, phong!({ color: 0xcfc8b8, specular: 0x262626, shininess: 40 })));
    }
    pinwheel.add(&wheel);
    pinwheel.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    pinwheel.rotation().y = rnd() * PI * 2.0;
    // (the old lean, within ±0.075): how far the wheel has turned on its pin
    // from the dowel, within ±20 degrees of the dowel lying under a side's
    // middle (where the sheet clears it; a drooping corner would not)
    let lean = (rnd() - 0.5) * 0.15;
    wheel.rotation().y = lean / 0.075 * 0.35;
    g.world.add(&pinwheel);
    // R419 30% chance: a draught along the floor stirs it on its pin, slowly
    // (a wheel lying on its corners cannot turn round), period 4-8 s.
    if rnd() < 0.30 {
        let spin_period = 4.0 + rnd() * 4.0;
        let spin_dir = if rnd() < 0.5 { -1.0 } else { 1.0 };
        let wg = wheel.clone();
        let base = wheel.rotation().y;
        g.animate(move |_g, _dt, t| {
            wg.rotation().y = base + spin_dir * 0.07 * (t * PI * 2.0 / spin_period).sin();
        });
    }
}

/// A vane's paper (sRGB `paper`): printed one side with rows of small white
/// dots, faded toward white unevenly, a tide line where it got wet and
/// dried, grey dirt from the floor, a crease, rubbed where it was handled
fn pinwheel_paper_tex(paper: u32) -> Tex {
    canvas_tex(128, 128, move |ctx, w, h| {
        let (r, g, b) = ((paper >> 16) & 255, (paper >> 8) & 255, paper & 255);
        ctx.set_fill_style(format!("rgb({r},{g},{b})"));
        ctx.fill_rect(0, 0, w, h);
        // the print: small dots in offset rows, not quite white (the ink was
        // cheap), some rubbed off
        for row in 0..9 {
            for col in 0..9 {
                let x = (col as f64 + if row % 2 == 0 { 0.25 } else { 0.75 }) * w / 9.0;
                let y = (row as f64 + 0.5) * h / 9.0;
                let a = 0.55 + rnd() * 0.25;
                if rnd() < 0.12 {
                    continue;
                }
                ctx.set_fill_style(format!("rgba(226,220,204,{a:.2})"));
                ctx.begin_path();
                ctx.arc(x, y, 2.6, 0, 7);
                ctx.fill();
            }
        }
        // faded unevenly toward the paper's own white
        for _ in 0..14 {
            let (x, y) = (rnd() * w, rnd() * h);
            let rr = 14.0 + rnd() * 30.0;
            let gr = ctx.create_radial_gradient(x, y, 0, x, y, rr);
            gr.add_color_stop(0, "rgba(232,226,210,0.28)");
            gr.add_color_stop(1, "rgba(232,226,210,0)");
            ctx.set_fill_style(gr);
            ctx.fill_rect(x - rr, y - rr, rr * 2.0, rr * 2.0);
        }
        // a tide line: it got wet and dried, its edge darker
        let (tx, ty, tr) = (rnd() * w, rnd() * h, 26.0 + rnd() * 22.0);
        ctx.set_stroke_style("rgba(92,76,52,0.30)");
        ctx.set_line_width(2.2);
        ctx.begin_path();
        ctx.arc(tx, ty, tr, 0, 7);
        ctx.stroke();
        ctx.set_fill_style("rgba(150,132,96,0.10)");
        ctx.begin_path();
        ctx.arc(tx, ty, tr, 0, 7);
        ctx.fill();
        // floor dirt: grey patches and specks
        for _ in 0..10 {
            let (x, y) = (rnd() * w, rnd() * h);
            let rr = 6.0 + rnd() * 16.0;
            let gr = ctx.create_radial_gradient(x, y, 0, x, y, rr);
            gr.add_color_stop(0, "rgba(70,64,56,0.22)");
            gr.add_color_stop(1, "rgba(70,64,56,0)");
            ctx.set_fill_style(gr);
            ctx.fill_rect(x - rr, y - rr, rr * 2.0, rr * 2.0);
        }
        for _ in 0..60 {
            ctx.set_fill_style(format!("rgba(60,54,46,{:.2})", 0.2 + rnd() * 0.35));
            ctx.fill_rect(rnd() * w, rnd() * h, 1.0 + rnd(), 1.0 + rnd());
        }
        // a crease across it, light on one side, dark on the other
        let (x0, y0) = (0.0, rnd() * h);
        let (x1, y1) = (w, rnd() * h);
        ctx.set_line_width(1.0);
        ctx.set_stroke_style("rgba(240,236,226,0.35)");
        ctx.begin_path();
        ctx.move_to(x0, y0);
        ctx.line_to(x1, y1);
        ctx.stroke();
        ctx.set_stroke_style("rgba(40,34,28,0.30)");
        ctx.begin_path();
        ctx.move_to(x0, y0 + 1.2);
        ctx.line_to(x1, y1 + 1.2);
        ctx.stroke();
    })
}

/// The birch dowel: pale wood gone grey-brown with handling and the floor,
/// its fibres in fine streaks, darker specks
fn pinwheel_dowel_tex() -> Tex {
    canvas_tex(64, 64, |ctx, w, h| {
        ctx.set_fill_style("#9a7d58");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..90 {
            let (x, y) = (rnd() * w, rnd() * h);
            let light = rnd() < 0.5;
            ctx.set_fill_style(if light { "rgba(196,170,128,0.30)" } else { "rgba(74,58,40,0.28)" });
            if rnd() < 0.5 {
                ctx.fill_rect(x, y, 4.0 + rnd() * 10.0, 1.0);
            } else {
                ctx.fill_rect(x, y, 1.0, 4.0 + rnd() * 10.0);
            }
        }
        for _ in 0..30 {
            ctx.set_fill_style("rgba(50,42,34,0.35)");
            ctx.fill_rect(rnd() * w, rnd() * h, 1, 1);
        }
    })
}

/// R417 small doll: 8% chance per level. Humanoid stick doll —
/// body cylinder + head sphere + 4 short limb cylinders.
pub fn place_small_dolls(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let body_colors = [0xd0b898, 0xa08060, 0x684030, 0xd8b8a0];
    let dress_colors = [0xd8a0c0, 0xa0c0d8, 0xc0d8a0, 0xd8c8a0, 0xa0a0d8];
    let body_mat = lambert!({ color: pick(&body_colors) });
    let dress_mat = lambert!({ color: pick(&dress_colors) });
    let eye_mat = basic_mat!({ color: 0x080604 });
    let doll = group();
    // Torso.
    let torso = mesh(pg("SmallDolls__dress", cyl_geo!(0.020, 0.020, 0.07, 8)), &dress_mat);
    torso.position().y = 0.075;
    doll.add(&torso);
    // Head sphere.
    let head = mesh(pg("SmallDolls__head", sphere_geo!(0.028, 8, 6)), &body_mat);
    head.position().y = 0.136; // chin 2 mm into the torso top (0.110)
    doll.add(&head);
    // 2 tiny dot eyes.
    for e_off in [-0.010, 0.010] {
        let eye = mesh(circle_geo!(0.0025, 6), &eye_mat);
        eye.position().set(e_off, 0.141, 0.026);
        doll.add(&eye);
    }
    // 2 arm cylinders sticking out.
    for a_off in [-0.030, 0.030] {
        let arm = mesh(pg("SmallDolls__arm", cyl_geo!(0.008, 0.008, 0.06, 6)), &body_mat);
        arm.rotation().z = PI / 2.0 + (if a_off > 0.0 { -0.4 } else { 0.4 });
        arm.position().set(a_off, 0.09, 0);
        doll.add(&arm);
    }
    // 2 leg cylinders extending down.
    for l_off in [-0.012, 0.012] {
        let leg = mesh(pg("SmallDolls__leg", cyl_geo!(0.009, 0.009, 0.06, 6)), &body_mat);
        leg.position().set(l_off, 0.03, 0);
        doll.add(&leg);
    }
    doll.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    doll.rotation().y = rnd() * PI * 2.0;
    // 40% chance doll lies on its back, face up, resting on the back of its head.
    if rnd() < 0.40 {
        doll.set_rotation_order(EulerOrder::YXZ);
        doll.rotation().x = -PI / 2.0;
        doll.position().y = 0.028;
    }
    g.world.add(&doll);
}

/// R416 small toy car: 10% chance per level. A 1:64 die-cast saloon of the
/// 1970s (Blender batch r17), 76 mm long: its zinc body in chipped enamel,
/// clear glazing gone grey, silver-painted bumpers, grille and lamps, thin
/// plastic wheels on steel axles, a black baseplate. A quarter lie on
/// their side.
pub fn place_toy_cars(g: &mut Game, _dist: &[i32]) {
    use game::assets::props::prop_part_as_modelled;
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // the old colours stand for the enamels die-cast cars came in, faded
    let body_colors = [0xd82020, 0x2050d8, 0xd8b820, 0x30a030, 0xd0d0d0, 0x181818];
    let col = pick(&body_colors);
    let k = body_colors.iter().position(|&c| c == col).unwrap_or(0);
    let paint = TOY_CAR_PAINT[k];
    let car = group();
    // the car in its own frame (x across, y up, z out of its nose)
    let body = group();
    let mut half_w: f64 = 0.015;
    let parts: [(&str, Mat); 6] = [
        ("Real__toyCarBody", phong!({ color: 0xffffff, map: &toy_car_paint_tex(paint), specular: 0x2c2c2c, shininess: 36 })),
        ("Real__toyCarGlass", phong!({ color: 0x23272a, specular: 0x4a4a4a, shininess: 70 })),
        ("Real__toyCarBright", phong!({ color: 0x7e7d78, specular: 0x5a5a5a, shininess: 48 })),
        ("Real__toyCarTail", lambert!({ color: 0x6e2a24 })),
        ("Real__toyCarBase", lambert!({ color: 0x24221f })),
        ("Real__toyCarTyres", lambert!({ color: 0x1f1e1c })),
    ];
    for (name, mat) in parts {
        if let Some(geo) = prop_part_as_modelled(name) {
            let bb = geo.bounding_box();
            half_w = half_w.max(bb.max.x.abs()).max(bb.min.x.abs());
            body.add(&mesh(geo, mat));
        }
    }
    car.add(&body);
    car.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    car.rotation().y = rnd() * PI * 2.0;
    // R417 25% chance: car on its side (rolled about its length), resting on
    // its flank.
    if rnd() < 0.25 {
        body.rotation().z = PI / 2.0 * (if rnd() < 0.5 { -1.0 } else { 1.0 });
        body.position().y = half_w;
    }
    g.world.add(&car);
}

/// The toy car's enamels (sRGB): red, blue, mustard, green, silver-grey,
/// black, as years and the light left them.
const TOY_CAR_PAINT: [u32; 6] = [0x8c2f28, 0x2f4878, 0xb08a32, 0x34583a, 0x8e908c, 0x2a2a2a];

/// The body's enamel (sRGB `paint`): a little uneven, dulled, chipped back
/// to the grey zinc in flecks and a few bigger chips (each with its dark
/// edge), fine scratches through it, dust in its hollows
fn toy_car_paint_tex(paint: u32) -> Tex {
    canvas_tex(128, 128, move |ctx, w, h| {
        let (r, g, b) = ((paint >> 16) & 255, (paint >> 8) & 255, paint & 255);
        ctx.set_fill_style(format!("rgb({r},{g},{b})"));
        ctx.fill_rect(0, 0, w, h);
        // dulled unevenly: lighter and darker washes
        for _ in 0..16 {
            let (x, y) = (rnd() * w, rnd() * h);
            let rr = 10.0 + rnd() * 26.0;
            let gr = ctx.create_radial_gradient(x, y, 0, x, y, rr);
            gr.add_color_stop(0, if rnd() < 0.5 { "rgba(200,196,186,0.14)" } else { "rgba(30,26,22,0.14)" });
            gr.add_color_stop(1, "rgba(0,0,0,0)");
            ctx.set_fill_style(gr);
            ctx.fill_rect(x - rr, y - rr, rr * 2.0, rr * 2.0);
        }
        // chips back to the zinc: a few bigger ones, ragged, with a dark
        // edge where the enamel broke; many tiny flecks
        for i in 0..22 {
            let (x, y) = (rnd() * w, rnd() * h);
            let big = i < 4;
            let r0 = if big { 1.6 + rnd() * 2.2 } else { 0.4 + rnd() * 0.6 };
            let n = 7;
            let pts: Vec<(f64, f64)> = (0..n)
                .map(|k| {
                    let a = k as f64 / n as f64 * PI * 2.0;
                    let rr = r0 * (0.5 + rnd() * 0.7);
                    (x + a.cos() * rr * 1.4, y + a.sin() * rr)
                })
                .collect();
            for (grow, style) in [(0.6, "rgba(34,32,30,0.45)"), (0.0, "rgba(146,146,140,0.9)")] {
                ctx.set_fill_style(style);
                ctx.begin_path();
                for (k, &(px, py)) in pts.iter().enumerate() {
                    let (dx, dy) = (px - x, py - y);
                    let d = (dx * dx + dy * dy).sqrt().max(1e-6);
                    let (qx, qy) = (px + dx / d * grow, py + dy / d * grow);
                    if k == 0 {
                        ctx.move_to(qx, qy);
                    } else {
                        ctx.line_to(qx, qy);
                    }
                }
                ctx.close_path();
                ctx.fill();
            }
        }
        // fine scratches, light
        ctx.set_line_width(0.6);
        for _ in 0..22 {
            let (x, y) = (rnd() * w, rnd() * h);
            let a = rnd() * PI;
            let l = 4.0 + rnd() * 14.0;
            ctx.set_stroke_style(format!("rgba(180,178,170,{:.2})", 0.25 + rnd() * 0.35));
            ctx.begin_path();
            ctx.move_to(x, y);
            ctx.line_to(x + a.cos() * l, y + a.sin() * l);
            ctx.stroke();
        }
        // dust and dirt specks
        for _ in 0..70 {
            ctx.set_fill_style(format!("rgba(96,88,74,{:.2})", 0.2 + rnd() * 0.3));
            ctx.fill_rect(rnd() * w, rnd() * h, 1, 1);
        }
    })
}

/// R415 marble scatter: 10% chance per level. 4-7 tiny colored
/// spheres scattered on floor — "spilled marbles".
pub fn place_marble_scatters(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let marble_colors = [0x2050d8, 0xd82020, 0xd8b820, 0x30a030, 0xa0a0a0, 0x8020d8];
    let cx = g.wx(x) + (rnd() - 0.5) * 2.0;
    let cz = g.wz(z) + (rnd() - 0.5) * 2.0;
    let count = 4 + (rnd() * 4.0).floor() as i32;
    for _ in 0..count {
        let m_col = pick(&marble_colors);
        let marble_mat = lambert!({ color: m_col });
        let marble = mesh(pg("MarbleScatters__marble", sphere_geo!(0.010, 6, 4)), &marble_mat);
        let m_ang = rnd() * PI * 2.0;
        let m_rad = rnd() * 0.30;
        marble.position().set(cx + m_ang.cos() * m_rad, 0.010, cz + m_ang.sin() * m_rad);
        g.world.add(&marble);
    }
}

/// R415 small building block: 10% chance per level. Colored cube with
/// 4 stud dots on top (LEGO-style silhouette).
pub fn place_building_blocks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let block_colors = [0xd82020, 0x2050d8, 0xd8b820, 0x30a030, 0xf0a020];
    let block_mat = lambert!({ color: pick(&block_colors) });
    let block = group();
    // Body: cube.
    let body = mesh(pg("BuildingBlocks__body", box_geo!(0.06, 0.03, 0.06)), &block_mat);
    body.position().y = 0.015;
    block.add(&body);
    // 4 stud dots on top (2×2 grid).
    for sx_off in [-0.015, 0.015] {
        for sz_off in [-0.015, 0.015] {
            let stud = mesh(pg("BuildingBlocks__stud", cyl_geo!(0.008, 0.008, 0.006, 6)), &block_mat);
            stud.position().set(sx_off, 0.033, sz_off);
            block.add(&stud);
        }
    }
    block.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    block.rotation().y = rnd() * PI * 2.0;
    g.world.add(&block);
    // R416 25% chance: 1-2 sibling blocks stacked on top.
    if rnd() < 0.25 {
        let stack_count = 1 + (rnd() * 2.0).floor() as i32;
        for s in 1..=stack_count {
            let sib_col = pick(&block_colors);
            let sib_mat = lambert!({ color: sib_col });
            let sib_block = block.duplicate();
            // Replace materials on all children.
            sib_block.traverse(&mut |child| {
                if child.is_mesh() {
                    child.set_mat(&sib_mat);
                }
            });
            sib_block.position().y = 0.036 * s as f64; // stack height per block
            sib_block.position().x += (rnd() - 0.5) * 0.015;
            sib_block.position().z += (rnd() - 0.5) * 0.015;
            sib_block.rotation().y = rnd() * PI * 2.0;
            g.world.add(&sib_block);
        }
    }
}

/// R414 small children's shoe: 10% chance per level. Small shoe
/// silhouette (elongated body + toe bump + sole).
pub fn place_children_shoes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let shoe_colors = [0xd82020, 0x2050d8, 0xf0f0f0, 0x181818, 0xd8b820];
    let shoe_mat = lambert!({ color: pick(&shoe_colors) });
    let sole_mat = lambert!({ color: 0x181410 });
    let shoe = group();
    // Body: elongated box.
    let body = mesh(pg("ChildrenShoes__upper", box_geo!(0.14, 0.05, 0.06)), &shoe_mat);
    body.position().y = 0.025;
    shoe.add(&body);
    // Toe bump: rounded box at front.
    let toe = mesh(pg("ChildrenShoes__toe", sphere_geo!(0.03, 6, 4)), &shoe_mat);
    toe.scale().set(1, 0.7, 1);
    toe.position().set(0.04, 0.025, 0); // ends flush with the sole's front (x 0.07)
    shoe.add(&toe);
    // Sole: thin dark box at bottom.
    let sole = mesh(pg("ChildrenShoes__sole", box_geo!(0.14, 0.008, 0.06)), &sole_mat);
    sole.position().y = 0.004;
    shoe.add(&sole);
    shoe.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    shoe.rotation().y = rnd() * PI * 2.0;
    g.world.add(&shoe);
}

/// R413 small crayon: 10% chance per level. Thin colored cylinder +
/// small pointed cone tip.
pub fn place_small_crayons(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let crayon_colors = [0xd82020, 0x2050d8, 0xd8b820, 0x30a030, 0x8020d8, 0xd05038, 0xd82060];
    let c_col = pick(&crayon_colors);
    let cray_mat = lambert!({ color: c_col });
    let crayon = group();
    // Body: thin cylinder.
    let body = mesh(pg("SmallCrayons__body", cyl_geo!(0.008, 0.008, 0.06, 6)), &cray_mat);
    body.rotation().z = PI / 2.0;
    crayon.add(&body);
    // Pointed tip: cone at one end.
    let tip = mesh(pg("SmallCrayons__tip", cone_geo!(0.008, 0.014, 6)), &cray_mat);
    tip.rotation().z = -PI / 2.0;
    tip.position().x = 0.037;
    crayon.add(&tip);
    crayon.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0.008, g.wz(z) + (rnd() - 0.5) * 2.4);
    crayon.rotation().y = rnd() * PI * 2.0;
    g.world.add(&crayon);
    // R414 30% chance: 1-3 sibling crayons scattered nearby.
    if rnd() < 0.30 {
        let sib_count = 1 + (rnd() * 3.0).floor() as i32;
        for _ in 0..sib_count {
            let sib_col = pick(&crayon_colors);
            let sib_mat = lambert!({ color: sib_col });
            let sib_crayon = group();
            let sib_body = mesh(pg("SmallCrayons__body", cyl_geo!(0.008, 0.008, 0.06, 6)), &sib_mat);
            sib_body.rotation().z = PI / 2.0;
            sib_crayon.add(&sib_body);
            let sib_tip = mesh(pg("SmallCrayons__tip", cone_geo!(0.008, 0.014, 6)), &sib_mat);
            sib_tip.rotation().z = -PI / 2.0;
            sib_tip.position().x = 0.037;
            sib_crayon.add(&sib_tip);
            let s_ang = rnd() * PI * 2.0;
            let s_rad = 0.06 + rnd() * 0.12;
            let cp = crayon.pos();
            sib_crayon.position().set(cp.x + s_ang.cos() * s_rad, 0.008, cp.z + s_ang.sin() * s_rad);
            sib_crayon.rotation().y = rnd() * PI * 2.0;
            g.world.add(&sib_crayon);
        }
    }
}

// R413 drawing paper texture — cream paper with random crayon-color
// scribble strokes. Built once.
game::lazy_static_tex!(
    drawing_paper_texture,
    canvas_tex_opt(96, 96, false, |ctx, w, h| {
        ctx.set_fill_style("#e8dcc0");
        ctx.fill_rect(0, 0, w, h);
        // Random crayon scribble strokes.
        let scribble_colors = ["#c82020", "#2050c8", "#c8b020", "#30a030", "#8020c8", "#d05038"];
        let strokes = 4 + (rnd() * 4.0).floor() as i32;
        for _ in 0..strokes {
            ctx.set_stroke_style(pick(&scribble_colors));
            ctx.set_line_width(2.0 + rnd() * 2.0);
            ctx.begin_path();
            let start_x = 10.0 + rnd() * (w - 20.0);
            let start_y = 10.0 + rnd() * (h - 20.0);
            ctx.move_to(start_x, start_y);
            // Wiggly line via 4 quadratic curves.
            let (mut cur_x, mut cur_y) = (start_x, start_y);
            for _ in 0..4 {
                let cp_x = cur_x + (rnd() - 0.5) * 30.0;
                let cp_y = cur_y + (rnd() - 0.5) * 30.0;
                let end_x = cur_x + (rnd() - 0.5) * 40.0;
                let end_y = cur_y + (rnd() - 0.5) * 40.0;
                ctx.quadratic_curve_to(cp_x, cp_y, end_x, end_y);
                cur_x = end_x;
                cur_y = end_y;
            }
            ctx.stroke();
        }
        // Slight grunge.
        for _ in 0..15 {
            ctx.set_fill_style(&format!("rgba(0,0,0,{})", 0.05 + rnd() * 0.08));
            ctx.fill_rect(rnd() * w, rnd() * h, 1, 1);
        }
    })
);

/// Drawing paper (R413): 10% chance per level.
pub fn place_drawing_papers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let paper_mat = lambert!({ map: drawing_paper_texture(), side: DoubleSide });
    let paper = mesh(plane_geo!(0.22, 0.28), &paper_mat);
    paper.rotation().x = -PI / 2.0;
    paper.rotation().z = rnd() * PI * 2.0;
    paper.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0.005, g.wz(z) + (rnd() - 0.5) * 2.4);
    g.world.add(&paper);
}

/// R412 small rubber ball: 10% chance per level. A 4 in hollow rubber ball
/// of two moulded halves, each its own colour, joined at a seam (Blender
/// batch r17); years on the floor have let the air out of it, so it sags
/// flat where it rests, and faded, scuffed and grimed it.
pub fn place_rubber_balls(g: &mut Game, _dist: &[i32]) {
    use game::assets::props::prop_part_as_modelled;
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // the old colours stand for the two halves' rubber, faded
    let ball_colors_a = [0xd82020, 0x2050d8, 0xd8b820, 0x30a030];
    let ball_colors_b = [0xf0f0f0, 0x181818, 0xd8b820, 0x8020d8];
    let c_a = pick(&ball_colors_a);
    let c_b = pick(&ball_colors_b);
    let ka = ball_colors_a.iter().position(|&c| c == c_a).unwrap_or(0);
    let kb = ball_colors_b.iter().position(|&c| c == c_b).unwrap_or(0);
    let rubber_a: [u32; 4] = [0x8e3a30, 0x3b5478, 0xb09048, 0x4d6d45];
    let rubber_b: [u32; 4] = [0xbcb5a3, 0x2c2a28, 0xa98a46, 0x5c4a68];
    let wear = rubber_ball_tex();
    let ball = group();
    for (name, col) in [("Real__rubberBallHalfA", rubber_a[ka]), ("Real__rubberBallHalfB", rubber_b[kb])] {
        if let Some(geo) = prop_part_as_modelled(name) {
            ball.add(&mesh(geo, phong!({ color: col, map: &wear, specular: 0x1c1c1c, shininess: 14 })));
        }
    }
    ball.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    ball.rotation().y = rnd() * PI * 2.0;
    g.world.add(&ball);
}

/// The rubber's wear (multiplies its colour): faded to a chalky bloom in
/// patches, grime rolled into it, scuffs where it hit things, specks of grit
fn rubber_ball_tex() -> Tex {
    canvas_tex(128, 128, |ctx, w, h| {
        ctx.set_fill_style("#e4dfd4");
        ctx.fill_rect(0, 0, w, h);
        // the bloom: rubber goes chalky and pale where the light found it
        for _ in 0..12 {
            let (x, y) = (rnd() * w, rnd() * h);
            let rr = 14.0 + rnd() * 30.0;
            let gr = ctx.create_radial_gradient(x, y, 0, x, y, rr);
            gr.add_color_stop(0, "rgba(255,252,244,0.35)");
            gr.add_color_stop(1, "rgba(255,252,244,0)");
            ctx.set_fill_style(gr);
            ctx.fill_rect(x - rr, y - rr, rr * 2.0, rr * 2.0);
        }
        // grime rolled in: soft dark patches
        for _ in 0..16 {
            let (x, y) = (rnd() * w, rnd() * h);
            let rr = 8.0 + rnd() * 22.0;
            let gr = ctx.create_radial_gradient(x, y, 0, x, y, rr);
            gr.add_color_stop(0, "rgba(64,56,44,0.22)");
            gr.add_color_stop(1, "rgba(64,56,44,0)");
            ctx.set_fill_style(gr);
            ctx.fill_rect(x - rr, y - rr, rr * 2.0, rr * 2.0);
        }
        // scuffs: short pale smears and dark scrapes
        for _ in 0..26 {
            let (x, y) = (rnd() * w, rnd() * h);
            let a = rnd() * PI;
            let l = 3.0 + rnd() * 9.0;
            ctx.set_line_width(1.0 + rnd() * 1.6);
            ctx.set_stroke_style(if rnd() < 0.6 { "rgba(250,246,236,0.35)" } else { "rgba(40,34,28,0.35)" });
            ctx.begin_path();
            ctx.move_to(x, y);
            ctx.line_to(x + a.cos() * l, y + a.sin() * l);
            ctx.stroke();
        }
        for _ in 0..90 {
            ctx.set_fill_style(format!("rgba(52,46,38,{:.2})", 0.25 + rnd() * 0.35));
            ctx.fill_rect(rnd() * w, rnd() * h, 1, 1);
        }
    })
}

/// R410 small handbasket: 10% chance per level. Shallow open box +
/// 2 arched handles. Reads as shopping-style handbasket.
pub fn place_handbaskets(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let body_colors = [0x8a2020, 0x2050a0, 0x686050, 0x2a2a2a];
    let body_mat = lambert!({ color: pick(&body_colors) });
    let inside_mat = basic_mat!({ color: 0x0a0806 });
    let handle_mat = lambert!({ color: 0x1a1610 });
    let basket = group();
    // Body: shallow open box.
    let body = mesh(pg("Handbaskets__body", box_geo!(0.22, 0.12, 0.16)), &body_mat);
    body.position().y = 0.06;
    basket.add(&body);
    // Inside dark disc (open top).
    let inside = mesh(plane_geo!(0.20, 0.14), &inside_mat);
    inside.rotation().x = -PI / 2.0;
    inside.position().y = 0.121;
    basket.add(&inside);
    // 2 arched handles across top: each half-ring (in XY, arching up) stands
    // on the rim of a long wall (z ±0.075), pivot hubs on the rim.
    for hx_off in [-0.09, 0.09] {
        let handle = mesh(pg("Handbaskets__handle", torus_geo!(0.05, 0.006, 4, 8, PI)), &handle_mat);
        handle.position().set(0, 0.118, hx_off * (0.075 / 0.09));
        basket.add(&handle);
    }
    basket.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    basket.rotation().y = rnd() * PI * 2.0;
    g.world.add(&basket);
}

/// R410 small suitcase: 10% chance per level. Larger box + top handle
/// + 2 side latches.
pub fn place_suitcases(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let body_mat = lambert!({ color: 0x4a2010 });
    let handle_mat = lambert!({ color: 0x2a1810 });
    let latch_mat = lambert!({ color: 0xa08040 });
    let suitcase = group();
    // Body: taller than briefcase.
    let body = mesh(pg("Suitcases__body", box_geo!(0.40, 0.28, 0.14)), &body_mat);
    body.position().y = 0.14;
    suitcase.add(&body);
    // Top handle: half-torus.
    let handle = mesh(pg("Suitcases__handle", torus_geo!(0.06, 0.008, 4, 8, PI)), &handle_mat);
    handle.rotation().z = 0.0; // arch over the top, end blocks on the lid
    handle.position().y = 0.286;
    suitcase.add(&handle);
    // 2 brass latches on top edge.
    for lx_off in [-0.12, 0.12] {
        let latch = mesh(pg("Suitcases__latch", box_geo!(0.03, 0.012, 0.024)), &latch_mat);
        latch.position().set(lx_off, 0.27, 0.075);
        suitcase.add(&latch);
    }
    // Small "sticker" plate on front-right.
    let sticker = mesh(pg("Suitcases__plate", plane_geo!(0.06, 0.04)), &latch_mat);
    sticker.position().set(0.10, 0.18, 0.0715); // plate back on the front face
    suitcase.add(&sticker);
    suitcase.position().set(g.wx(x) + (rnd() - 0.5) * 2.2, 0, g.wz(z) + (rnd() - 0.5) * 2.2);
    suitcase.rotation().y = rnd() * PI * 2.0;
    g.world.add(&suitcase);
}

/// R409 wall paper towel dispenser: 12% chance per level. Tall
/// gray box + horizontal dispensing slot + small hanging paper end.
pub fn place_paper_towel_dispensers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.88 {
        return;
    } // 12% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let (cx, cz, cdx, cdz) = pick(&candidates);
    let body_mat = lambert!({ color: 0xa0a098 });
    let slot_mat = basic_mat!({ color: 0x0a0806 });
    let paper_mat = lambert!({ color: 0xf0e8d0 });
    let ptd = group();
    // Body: tall box.
    let body = mesh(pg("PaperTowelDispensers__body", box_geo!(0.24, 0.34, 0.10)), &body_mat);
    body.position().y = 0.0;
    ptd.add(&body);
    // Dispensing slot: dark horizontal recess near bottom.
    let slot = mesh(plane_geo!(0.18, 0.02), &slot_mat);
    slot.position().set(0, -0.15, 0.051);
    ptd.add(&slot);
    // Small paper towel hanging out of slot: top edge inside the slot recess,
    // hanging out over its lip.
    let paper_end = mesh(pg("PaperTowelDispensers__towel", plane_geo!(0.10, 0.06)), &paper_mat);
    paper_end.position().set(0, -0.178, 0.053);
    paper_end.rotation().x = -0.15;
    ptd.add(&paper_end);
    let wall_x = g.wx(cx) + cdx as f64 * (CELL * 0.5 - 0.05);
    let wall_z = g.wz(cz) + cdz as f64 * (CELL * 0.5 - 0.05);
    let ptd_height = 1.4 + rnd() * 0.15;
    if cdx.abs() == 1 {
        ptd.rotation().y = if cdx > 0 { -PI / 2.0 } else { PI / 2.0 };
        ptd.position().set(wall_x, ptd_height, wall_z + (rnd() - 0.5) * 0.6);
    } else {
        ptd.rotation().y = if cdz > 0 { PI } else { 0.0 };
        ptd.position().set(wall_x + (rnd() - 0.5) * 0.6, ptd_height, wall_z);
    }
    g.world.add(&ptd);
}

/// R409 tissue box: 12% chance per level. A flat box of facial tissues
/// (Blender batch r15, the 160-sheet carton at its real size, 225 x 121 x
/// 88 mm): its printed board, the oval torn out along its perforation, a
/// top corner trodden down, the last tissue drawn up gone limp over the
/// top, all of it dusty. The old draws are kept call for call.
pub fn place_tissue_boxes(g: &mut Game, _dist: &[i32]) {
    use game::assets::props::prop_part_as_modelled;
    if rnd() < 0.88 {
        return;
    } // 12% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // the old colour draw picks the carton's print: cream, pale blue, lilac, grey
    let box_colors = [0xe8dcc0, 0xa0d0e8, 0xd0a0d8, 0xa0a0a0];
    let col = pick(&box_colors);
    let look = box_colors.iter().position(|&c| c == col).unwrap_or(0);
    // the old tissue's tilt: now the limp tissue's turn in the slot
    let lean = (rnd() - 0.5) * 0.3;
    let tissue = group();
    let px = g.wx(x) + (rnd() - 0.5) * 2.4;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.4;
    tissue.position().set(px, 0, pz);
    tissue.rotation().y = rnd() * PI * 2.0;
    // what is printed and how worn differs floor to floor: from where it lies
    let salt = r15_tissue::salt(px, pz);
    let atlas = r15_tissue::carton_atlas(look, salt);
    let cell = |c: (f64, f64, f64, f64)| {
        let t = atlas.duplicate();
        let (w, h) = (r15_tissue::ATLAS.0, r15_tissue::ATLAS.1);
        t.set_offset((c.0 + 1.0) / w, 1.0 - (c.1 + c.3 - 1.0) / h);
        t.set_repeat((c.2 - 2.0) / w, (c.3 - 2.0) / h);
        lambert!({ map: &t })
    };
    let parts = [
        ("Real__tissueBoxTop", cell(r15_tissue::TOP)),
        ("Real__tissueBoxSides", cell(r15_tissue::SIDE)),
        ("Real__tissueBoxEnds", cell(r15_tissue::END)),
        ("Real__tissueBoxInside", lambert!({ color: 0x7e776a })),
        ("Real__tissueBoxStack", lambert!({ color: 0xcfcabd })),
    ];
    for (name, mat) in &parts {
        if let Some(geo) = prop_part_as_modelled(name) {
            tissue.add(&mesh(geo, mat));
        }
    }
    if let Some(geo) = prop_part_as_modelled("Real__tissueBoxSheet") {
        let sheet = mesh(geo, &lambert!({ map: &r15_tissue::sheet_tex(salt), side: DoubleSide }));
        sheet.rotation().y = lean;
        tissue.add(&sheet);
    }
    g.world.add(&tissue);
}

/// R408 wall electric hand dryer: 12% chance per level. Large white
/// box + curved bottom nozzle — restroom institutional fixture.
pub fn place_hand_dryers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.88 {
        return;
    } // 12% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let (cx, cz, cdx, cdz) = pick(&candidates);
    let body_mat = lambert!({ color: 0xd0c8b8 });
    let nozzle_mat = lambert!({ color: 0x2a2418 });
    let dryer = group();
    // Main body: box.
    let body = mesh(pg("HandDryers__body", box_geo!(0.32, 0.30, 0.14)), &body_mat);
    body.position().y = 0.0;
    dryer.add(&body);
    // Curved bottom nozzle: short slanted box at bottom.
    let nozzle = mesh(pg("HandDryers__nozzle", box_geo!(0.12, 0.04, 0.06)), &nozzle_mat);
    nozzle.rotation().x = -0.5;
    nozzle.position().set(0, -0.17, 0.06);
    dryer.add(&nozzle);
    // Small round emergency button on right side.
    let btn = mesh(pg("HandDryers__button", cyl_geo!(0.015, 0.015, 0.005, 8)), &nozzle_mat);
    btn.rotation().x = PI / 2.0;
    btn.position().set(0.12, 0.05, 0.071);
    dryer.add(&btn);
    let wall_x = g.wx(cx) + cdx as f64 * (CELL * 0.5 - 0.07);
    let wall_z = g.wz(cz) + cdz as f64 * (CELL * 0.5 - 0.07);
    let dryer_height = 1.2 + rnd() * 0.15;
    if cdx.abs() == 1 {
        dryer.rotation().y = if cdx > 0 { -PI / 2.0 } else { PI / 2.0 };
        dryer.position().set(wall_x, dryer_height, wall_z + (rnd() - 0.5) * 0.6);
    } else {
        dryer.rotation().y = if cdz > 0 { PI } else { 0.0 };
        dryer.position().set(wall_x + (rnd() - 0.5) * 0.6, dryer_height, wall_z);
    }
    g.world.add(&dryer);
}

/// The tissue box's print and wear (`place_tissue_boxes`).
mod r15_tissue {
    use game::prelude::*;

    /// The carton's atlas (px) and its cells (x, y, w, h): the top panel
    /// (225 x 121 mm), a long side (225 x 88 mm, both sides print alike), an
    /// end (121 x 88 mm)
    pub const ATLAS: (f64, f64) = (1024.0, 512.0);
    pub const TOP: (f64, f64, f64, f64) = (0.0, 0.0, 560.0, 302.0);
    pub const SIDE: (f64, f64, f64, f64) = (0.0, 302.0, 560.0, 210.0);
    pub const END: (f64, f64, f64, f64) = (560.0, 0.0, 300.0, 216.0);

    /// A salt from where it lies
    pub fn salt(px: f64, pz: f64) -> u32 {
        ((px * 977.0).floor() as i64 ^ ((pz * 1181.0).floor() as i64).rotate_left(13)) as u32
    }

    /// The print's ground and its pattern's ink, by the old colour (sRGB):
    /// cream with grey-green sprigs, pale blue with white waves, lilac
    /// with white flowers, grey with a fine stripe
    const GROUND: [(f64, f64, f64); 4] = [(214.0, 202.0, 178.0), (146.0, 170.0, 182.0), (170.0, 148.0, 172.0), (156.0, 154.0, 150.0)];
    const INK: [(f64, f64, f64); 4] = [(132.0, 146.0, 120.0), (226.0, 230.0, 228.0), (228.0, 222.0, 226.0), (112.0, 112.0, 116.0)];

    fn rgba(c: (f64, f64, f64), a: f64) -> String {
        format!("rgba({:.0},{:.0},{:.0},{a:.3})", c.0, c.1, c.2)
    }

    /// A soft round spot: `r` px, `k` times as tall as wide, turned `turn`
    /// (the context squashed, so the fade is never cut by an edge)
    fn soft(ctx: &mut Canvas, at: (f64, f64), r: f64, k: f64, turn: f64, c: (f64, f64, f64), a: f64) {
        ctx.save();
        ctx.translate(at.0, at.1);
        ctx.rotate(turn);
        ctx.scale(1, k);
        let gr = ctx.create_radial_gradient(0, 0, 0, 0, 0, r);
        gr.add_color_stop(0, &rgba(c, a));
        gr.add_color_stop(1, &rgba(c, 0.0));
        ctx.set_fill_style(&gr);
        ctx.begin_path();
        ctx.arc(0, 0, r, 0, 7);
        ctx.fill();
        ctx.restore();
    }

    /// The pattern's repeat, drawn once and laid like a printing cylinder's
    /// (sprigs in a half-drop, waves, five-petalled flowers); the grey
    /// look's stripe is drawn straight
    fn repeat_tile(look: usize) -> Option<(Canvas, f64, f64)> {
        let ink = INK[look];
        let (tw, th) = match look {
            0 => (72.0, 60.0),
            1 => (48.0, 28.0),
            2 => (52.0, 52.0),
            _ => return None,
        };
        let mut t = Canvas::new(tw as u32, th as u32);
        let wraps = |cx: f64, cy: f64| {
            let mut v = vec![];
            for ox in [-tw, 0.0, tw] {
                for oy in [-th, 0.0, th] {
                    v.push((cx + ox, cy + oy));
                }
            }
            v
        };
        match look {
            0 => {
                for (k, (mx, my)) in [(18.0, 15.0), (54.0, 45.0)].into_iter().enumerate() {
                    for (cx, cy) in wraps(mx, my) {
                        t.save();
                        t.translate(cx, cy);
                        t.rotate(-0.42 + 0.18 * k as f64);
                        t.set_stroke_style(rgba(ink, 0.7));
                        t.set_line_width(1.0);
                        t.begin_path();
                        t.move_to(0, 7);
                        t.quadratic_curve_to(1.5, 0, 0, -8);
                        t.stroke();
                        t.set_fill_style(rgba(ink, 0.65));
                        for (ly, sd) in [(-5.0, 1.0), (-1.0, -1.0), (3.0, 1.0), (-8.0, -1.0)] {
                            t.save();
                            t.translate(0, ly);
                            t.rotate(sd * 0.9);
                            t.scale(1, 0.42);
                            t.begin_path();
                            t.arc(sd * 3.4, 0, 3.4, 0, 7);
                            t.fill();
                            t.restore();
                        }
                        t.restore();
                    }
                }
            }
            1 => {
                t.set_stroke_style(rgba(ink, 0.55));
                t.set_line_width(1.6);
                for y in [7.0, 21.0] {
                    t.begin_path();
                    let mut x = -24.0;
                    t.move_to(x, y);
                    while x < tw + 24.0 {
                        t.quadratic_curve_to(x + 6.0, y - 4.5, x + 12.0, y);
                        t.quadratic_curve_to(x + 18.0, y + 4.5, x + 24.0, y);
                        x += 24.0;
                    }
                    t.stroke();
                }
            }
            _ => {
                for (k, (mx, my)) in [(13.0, 13.0), (39.0, 39.0)].into_iter().enumerate() {
                    for (cx, cy) in wraps(mx, my) {
                        let rot = 0.3 + 0.5 * k as f64;
                        t.set_fill_style(rgba(ink, 0.7));
                        for p in 0..5 {
                            let a = rot + p as f64 * PI * 2.0 / 5.0;
                            t.begin_path();
                            t.arc(cx + a.cos() * 3.2, cy + a.sin() * 3.2, 2.4, 0, 7);
                            t.fill();
                        }
                        t.set_fill_style(rgba((196.0, 170.0, 110.0), 0.8));
                        t.begin_path();
                        t.arc(cx, cy, 1.5, 0, 7);
                        t.fill();
                    }
                }
            }
        }
        Some((t, tw, th))
    }

    /// The pattern printed all over a cell, by the look
    fn pattern(ctx: &mut Canvas, look: usize, c: (f64, f64, f64, f64), tile: &Option<(Canvas, f64, f64)>) {
        let (x0, y0, w, h) = c;
        ctx.save();
        ctx.begin_path();
        ctx.rect(x0, y0, w, h);
        ctx.clip();
        if let Some((t, tw, th)) = tile {
            let mut y = y0;
            while y < y0 + h {
                let mut x = x0;
                while x < x0 + w {
                    ctx.draw_image(t, x, y);
                    x += tw;
                }
                y += th;
            }
        } else {
            ctx.set_fill_style(rgba(INK[look], 0.35));
            let mut x = x0 - h;
            while x < x0 + w {
                ctx.begin_path();
                ctx.move_to(x, y0 + h);
                ctx.line_to(x + h, y0);
                ctx.line_to(x + h + 2.5, y0);
                ctx.line_to(x + 2.5, y0 + h);
                ctx.close_path();
                ctx.fill();
                x += 7.0;
            }
        }
        ctx.restore();
    }

    /// Board worn at a cell's edges (the folds rubbed to the grey board
    /// under the print) and dust lying on it
    fn worn_edges(ctx: &mut Canvas, c: (f64, f64, f64, f64), r: &mut SplitMix64) {
        let (x0, y0, w, h) = c;
        for _ in 0..90 {
            let side = (r.next() * 4.0) as i32;
            let t = r.next();
            let (x, y) = match side {
                0 => (x0 + t * w, y0 + r.next() * 3.0),
                1 => (x0 + t * w, y0 + h - r.next() * 3.0),
                2 => (x0 + r.next() * 3.0, y0 + t * h),
                _ => (x0 + w - r.next() * 3.0, y0 + t * h),
            };
            let l = 3.0 + r.next() * 16.0;
            ctx.set_fill_style(rgba((198.0, 192.0, 180.0), 0.25 + 0.35 * r.next()));
            if side < 2 {
                ctx.fill_rect(x, y - 0.8, l, 1.6);
            } else {
                ctx.fill_rect(x - 0.8, y, 1.6, l);
            }
        }
    }

    /// The carton's atlas: the top's pattern and the oval's torn edge, the
    /// sides' label in a serif and the pattern, the end's count and its
    /// bar code; worn at every fold, dusty, the corner that was trodden on
    /// creased, a tide line where water stood on the floor
    pub fn carton_atlas(look: usize, salt: u32) -> Tex {
        canvas_tex_opt(ATLAS.0, ATLAS.1, false, move |ctx, w, h| {
            let mut r = SplitMix64::new(salt, 0x7155_0e5b);
            let g = GROUND[look];
            ctx.set_fill_style(rgba(g, 1.0));
            ctx.fill_rect(0, 0, w, h);
            // the top: the pattern, a printed ring round the oval, the
            // oval's perforation torn, the maker's words at one end
            let tile = repeat_tile(look);
            pattern(ctx, look, TOP, &tile);
            let (cx, cy) = (TOP.2 / 2.0, TOP.3 / 2.0);
            let (rx, ry) = (TOP.2 * 0.056 / 0.225, TOP.3 * 0.021 / 0.121);
            ctx.set_fill_style(rgba((236.0, 232.0, 222.0), 0.9));
            ctx.begin_path();
            ctx.ellipse(cx, cy, rx + 16.0, ry + 14.0, 0, 0, 7);
            ctx.fill();
            ctx.set_stroke_style(rgba(INK[look], 0.8));
            ctx.set_line_width(1.4);
            ctx.begin_path();
            ctx.ellipse(cx, cy, rx + 12.0, ry + 10.0, 0, 0, 7);
            ctx.stroke();
            // the torn perforation: the board's grey core showing at the edge
            for k in 0..160 {
                let a = k as f64 / 160.0 * PI * 2.0;
                let f = 1.0 + 0.04 * r.next();
                ctx.set_fill_style(rgba((172.0, 166.0, 154.0), 0.5 + 0.4 * r.next()));
                ctx.fill_rect(cx + a.cos() * rx * f - 1.0, cy + a.sin() * ry * f - 1.0, 1.5 + r.next() * 2.0, 1.5 + r.next() * 1.5);
            }
            ctx.set_fill_style(rgba((84.0, 78.0, 70.0), 0.75));
            ctx.set_text_align("left");
            ctx.set_font("italic 15px Times New Roman");
            ctx.fill_text("facial tissues", 18, TOP.3 - 18.0);
            worn_edges(ctx, TOP, &mut r);
            // the side: the pattern, a label panel and its words
            pattern(ctx, look, SIDE, &tile);
            let (sx, sy) = (SIDE.0 + 70.0, SIDE.1 + 38.0);
            ctx.set_fill_style(rgba((234.0, 230.0, 220.0), 0.94));
            ctx.begin_path();
            ctx.move_to(sx + 10.0, sy);
            ctx.arc_to(sx + 250.0, sy, sx + 250.0, sy + 120.0, 10);
            ctx.arc_to(sx + 250.0, sy + 120.0, sx, sy + 120.0, 10);
            ctx.arc_to(sx, sy + 120.0, sx, sy, 10);
            ctx.arc_to(sx, sy, sx + 250.0, sy, 10);
            ctx.close_path();
            ctx.fill();
            ctx.set_fill_style(rgba((70.0, 64.0, 60.0), 0.92));
            ctx.set_text_align("center");
            ctx.set_font("italic 34px Times New Roman");
            ctx.fill_text("Facial Tissues", sx + 125.0, sy + 50.0);
            ctx.set_font("15px Gill Sans");
            ctx.fill_text("SOFT  ·  2-PLY  ·  UNSCENTED", sx + 125.0, sy + 78.0);
            ctx.set_font("bold 17px Gill Sans");
            ctx.fill_text("160 TISSUES", sx + 125.0, sy + 103.0);
            ctx.set_font("11px Gill Sans");
            ctx.set_fill_style(rgba((70.0, 64.0, 60.0), 0.8));
            ctx.fill_text("2-ply facial tissue  ·  20.8 cm x 21.3 cm", SIDE.0 + 420.0, SIDE.1 + SIDE.3 - 14.0);
            worn_edges(ctx, SIDE, &mut r);
            // the end: the count, the size, the bar code
            ctx.set_fill_style(rgba((234.0, 230.0, 220.0), 0.94));
            ctx.fill_rect(END.0 + 22.0, END.1 + 24.0, END.2 - 44.0, 104);
            ctx.set_fill_style(rgba((70.0, 64.0, 60.0), 0.92));
            ctx.set_font("bold 40px Gill Sans");
            ctx.fill_text("160", END.0 + END.2 / 2.0, END.1 + 70.0);
            ctx.set_font("13px Gill Sans");
            ctx.fill_text("2-PLY FACIAL TISSUES", END.0 + END.2 / 2.0, END.1 + 92.0);
            ctx.set_font("11px Gill Sans");
            ctx.fill_text("80 double sheets  ·  20.8 x 21.3 cm", END.0 + END.2 / 2.0, END.1 + 114.0);
            // the bar code: an EAN-13's guard bars and modules over its digits
            let (bx, by) = (END.0 + 92.0, END.1 + 138.0);
            ctx.set_fill_style("rgba(236,232,224,0.95)");
            ctx.fill_rect(bx - 8.0, by - 4.0, 132, 42);
            ctx.set_fill_style("rgba(30,28,26,0.9)");
            let mut xx = bx;
            for k in 0..95 {
                let guard = k < 3 || (45..50).contains(&k) || k >= 92;
                let on = if guard { k % 2 == 0 || k == 46 || k == 48 } else { r.next() < 0.5 };
                if on {
                    ctx.fill_rect(xx, by, 1.2, if guard { 32.0 } else { 28.0 });
                }
                xx += 1.2;
            }
            ctx.set_font("9px Helvetica");
            ctx.set_text_align("left");
            let digits: String = (0..13).map(|_| char::from(b'0' + (r.next() * 10.0) as u8)).collect();
            ctx.fill_text(&digits[..1], bx - 7.0, by + 37.0);
            ctx.fill_text(&digits[1..7], bx + 5.0, by + 37.0);
            ctx.fill_text(&digits[7..], bx + 60.0, by + 37.0);
            // the outer flap's edge, 14.5 mm above the foot: its shadow line
            let fy = END.1 + END.3 * (1.0 - 0.0145 / 0.088);
            ctx.set_fill_style("rgba(40,34,28,0.35)");
            ctx.fill_rect(END.0, fy, END.2, 1.5);
            ctx.set_fill_style("rgba(250,246,236,0.18)");
            ctx.fill_rect(END.0, fy - 1.5, END.2, 1.5);
            worn_edges(ctx, END, &mut r);
            // the corner trodden on (+x, +z): creases out from it on the top
            // (its right, nearest edge) and on the long side (its top right)
            ctx.set_line_width(1.0);
            for (ox, oy, a0) in [(TOP.2, TOP.3, PI), (SIDE.2, SIDE.1, PI * 0.5)] {
                for _ in 0..9 {
                    let a = a0 + 0.1 + r.next() * (PI * 0.5 - 0.2);
                    let l = 18.0 + r.next() * 50.0;
                    let (sx0, sy0) =
                        (ox - 4.0 * r.next(), oy + if oy < SIDE.1 + 1.0 && oy > TOP.3 { 4.0 * r.next() } else { -4.0 * r.next() });
                    ctx.set_stroke_style(rgba((60.0, 52.0, 44.0), 0.35));
                    ctx.begin_path();
                    ctx.move_to(sx0, sy0);
                    ctx.line_to(sx0 + a.cos() * l, sy0 + a.sin() * l);
                    ctx.stroke();
                    ctx.set_stroke_style(rgba((240.0, 234.0, 222.0), 0.25));
                    ctx.begin_path();
                    ctx.move_to(sx0 + 1.0, sy0 + 1.0);
                    ctx.line_to(sx0 + a.cos() * l + 1.0, sy0 + a.sin() * l + 1.0);
                    ctx.stroke();
                }
            }
            // the floor's grime up the sides and ends, a tide line where
            // water stood round it once
            for c in [SIDE, END] {
                let (x0, y0, cw, ch) = c;
                let gr = ctx.create_linear_gradient(0, y0 + ch, 0, y0 + ch - 40.0);
                gr.add_color_stop(0, "rgba(70,60,46,0.55)");
                gr.add_color_stop(1, "rgba(70,60,46,0)");
                ctx.set_fill_style(&gr);
                ctx.fill_rect(x0, y0 + ch - 40.0, cw, 40);
                let tide = y0 + ch - 22.0 - 6.0 * r.next();
                ctx.set_stroke_style("rgba(110,86,52,0.45)");
                ctx.set_line_width(1.6);
                ctx.begin_path();
                let mut xx = x0;
                ctx.move_to(xx, tide);
                while xx < x0 + cw {
                    xx += 6.0;
                    ctx.line_to(xx, tide + (r.next() - 0.5) * 4.0);
                }
                ctx.stroke();
                let gr = ctx.create_linear_gradient(0, tide, 0, y0 + ch);
                gr.add_color_stop(0, "rgba(120,96,60,0.18)");
                gr.add_color_stop(1, "rgba(120,96,60,0.05)");
                ctx.set_fill_style(&gr);
                ctx.fill_rect(x0, tide, cw, y0 + ch - tide);
            }
            // dust: a film over all of it, heaviest on the top, in drifts
            ctx.set_fill_style("rgba(120,110,94,0.20)");
            ctx.fill_rect(0, 0, w, h);
            for _ in 0..36 {
                let at = (TOP.0 + r.next() * TOP.2, TOP.1 + r.next() * TOP.3);
                soft(ctx, at, 20.0 + r.next() * 50.0, 0.5 + r.next() * 0.5, r.next() * 3.0, (128.0, 118.0, 100.0), 0.10 + 0.14 * r.next());
            }
            ctx.set_fill_style("rgba(128,118,100,0.16)");
            ctx.fill_rect(TOP.0, TOP.1, TOP.2, TOP.3);
            for _ in 0..260 {
                let (x, y) = (r.next() * w, r.next() * h);
                ctx.set_fill_style(rgba((90.0, 82.0, 70.0), 0.15 + 0.25 * r.next()));
                ctx.fill_rect(x, y, 1.0 + r.next() * 1.5, 1.0 + r.next() * 1.5);
            }
        })
    }

    /// The limp tissue: soft white gone grey with dust, darker in its folds
    pub fn sheet_tex(salt: u32) -> Tex {
        canvas_tex_opt(128, 128, false, move |ctx, w, h| {
            let mut r = SplitMix64::new(salt, 0x5ee7);
            ctx.set_fill_style("#c8c2b4");
            ctx.fill_rect(0, 0, w, h);
            for _ in 0..40 {
                let at = (r.next() * w, r.next() * h);
                soft(ctx, at, 6.0 + r.next() * 20.0, 0.4 + r.next() * 0.6, r.next() * 3.0, (120.0, 110.0, 94.0), 0.12 + 0.2 * r.next());
            }
            // the embossed dots of two-ply tissue, faint
            ctx.set_fill_style("rgba(255,255,250,0.12)");
            let mut y = 2.0;
            while y < h {
                let mut x = if (y as i32 / 6) % 2 == 0 { 2.0 } else { 5.0 };
                while x < w {
                    ctx.fill_rect(x, y, 1.2, 1.2);
                    x += 6.0;
                }
                y += 6.0;
            }
        })
    }
}
