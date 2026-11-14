//! `place_cracked_wall_holes`
use game::prelude::*;
use placers_kit::on_wall::mix01;
use placers_kit::on_wall::wall_face;
use placers_kit::on_wall::wall_turn;
use placers_kit::place::shuffle;
use placers_kit::place::wall_candidates;
use placers_kit::place::WallCand;
use placers_kit::wall_plates::part;

// Cracked wall segments: a hole kicked or punched through the wall
// (Blender batch `r06`): the face paper torn back, the gypsum broken through
// its 12.7 mm, chunks hanging in on their paper, the stud bay behind it (a
// steel stud, the back of the far board), cracks out from it, the crumbs on
// the floor below. The wall is not cut: the hole's mask is drawn first and
// writes depth only, so the wall is not drawn inside it.
pub(crate) fn place_cracked_wall_holes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.65 {
        return;
    } // 35% chance
    let count = 1 + (rnd() * 2.0).floor() as usize;
    let mut candidates = wall_candidates(g);
    shuffle(&mut candidates);
    let mut i = 0;
    while i < count && i < candidates.len() {
        let c = candidates[i];
        // its size: 0.10-0.22 wide x 0.08-0.18 tall (the same draws as ever)
        let w = 0.10 + rnd() * 0.12;
        let hole_h = 0.08 + rnd() * 0.10;
        let h_height = 0.6 + rnd() * 1.4; // 0.6 - 2.0 m (waist to head)
        let (rot, along) = wall_turn(&c);
        let jit = (rnd() - 0.5) * 1.4;
        let (wx, wz) = wall_face(g, &c);
        let hole = build_wall_hole(g, &c, w, hole_h, h_height, jit);
        hole.rotation().y = rot;
        hole.position().set(wx + along.0 * jit, h_height, wz + along.1 * jit);
        g.world.add(&hole);
        i += 1;
    }
}

/// What the wall behind candidate `c` is made of: its arch material's key.
fn wall_key_of(g: &Game, c: &WallCand) -> String {
    let (nx, nz) = (c.x + c.dx, c.z + c.dz);
    if !g.in_grid(nx, nz) || g.layout.is_none() {
        return "wp:0".into();
    }
    g.wall_mat_key(g.t_idx(nx, nz))
}

/// The hole: its group's origin on the wall's face at the hole's centre,
/// +z out of the wall, y up (the floor at -h_height).
fn build_wall_hole(g: &mut Game, c: &WallCand, w: f64, hole_h: f64, h_height: f64, jit: f64) -> Obj {
    // the board (1/2 in) and the bay of a 3-5/8 in stud behind it
    const BOARD: f64 = 0.0127;
    const BAY: f64 = 0.092;
    // the size: the nearest of the three modelled holes, scaled a little
    let size = (w * hole_h).sqrt();
    let kinds = [("A", 0.1015), ("B", 0.1373), ("C", 0.1833)];
    let (k, nominal) = kinds.iter().copied().min_by(|a, b| (a.1 - size).abs().total_cmp(&(b.1 - size).abs())).unwrap();
    let scale = (size / nominal).clamp(0.85, 1.15);
    let key = wall_key_of(g, c);
    let masonry = !(key.starts_with("wp:") || key.starts_with("drywall") || key.starts_with("paint"));
    let root = group();
    // it is in the wall on purpose: the placement pass must not push it out
    // (the settle pass still slides it onto the wall's real face)
    root.user().floats = true;
    let hg = group();
    hg.scale().set(scale, scale, scale);
    root.add(&hg);
    // drawn before the walls: what is inside the wall, then the mask
    let inside = |o: &Obj| {
        o.set_render_order(-3);
    };
    // what the broken edge is: gypsum, or the masonry the wall is built of
    let edge = if !masonry {
        0xaba496
    } else if key.starts_with("brick") {
        0x5e3428
    } else if key.starts_with("tile") {
        0xa29d91
    } else {
        0x77736b
    };
    let gypsum = lambert!({ color: edge });
    let gy = mesh(part(&format!("Real__hole{k}Gypsum"), 0.2, 0.15, 0.013), &gypsum);
    inside(&gy);
    hg.add(&gy);
    if !masonry {
        // the face paper: the wall's own surface, torn and pushed in, in the
        // shade of the hole
        let paper_mat = g.arch_mat(&key).duplicate();
        paper_mat.set_color(0x6a6a6a);
        let pa = mesh(part(&format!("Real__hole{k}Paper"), 0.2, 0.15, 0.01), &paper_mat);
        inside(&pa);
        hg.add(&pa);
        // the stud bay: a steel stud where the hole happens to be by one
        let off = -0.20 + mix01(jit, h_height) * 0.40;
        let stud_mat = lambert!({ color: 0x3c3e3f });
        for sx in [-0.406, 0.0, 0.406] {
            let x = off + sx;
            if x.abs() > 0.38 {
                continue;
            }
            let st = mesh(part("Real__holeStud", 0.04, 0.5, 0.092), &stud_mat);
            st.position().x = x;
            inside(&st);
            hg.add(&st);
        }
    }
    // the back of the bay: the far board's paper back (or the rough core
    // of a block), dark away from the light coming in through the hole
    let back_depth = if masonry { 0.035 } else { BOARD + BAY };
    let back_tint = if key.starts_with("brick") { 0xb07060 } else { 0xffffff };
    let back = mesh(plane_geo!(0.8, 0.5), lambert!({ map: &hole_back_tex(masonry), color: back_tint }));
    back.position().z = -back_depth;
    inside(&back);
    hg.add(&back);
    // the mask: the torn outline, writing depth only
    let mask = mesh(part(&format!("Real__hole{k}Mask"), 0.2, 0.15, 0.001), basic_mat!({ colorWrite: false }));
    mask.set_render_order(-2);
    hg.add(&mask);
    // the cracks out from it, the dent, the scuffs: on the wall, behind the mask
    let ki = match k {
        "A" => 0,
        "B" => 1,
        _ => 2,
    };
    // what is inside the wall (the broken edge, the paper pushed in, the
    // bay, the mask) is the wall's own inside, not a thing stood in it
    for k in 0..hg.child_count() {
        hg.child(k).unwrap().user().arch = true;
    }
    let decal = mesh(plane_geo!(0.56, 0.56), lambert!({ map: &hole_decal_tex(ki), transparent: true, depthWrite: false }));
    decal.position().z = 0.0008;
    hg.add(&decal);
    // what fell out of it, on the floor at the wall's foot
    let chips = mesh(part("Real__holeChips", 0.25, 0.02, 0.12), &gypsum);
    chips.position().set(0, -h_height, 0.01);
    root.add(&chips);
    let dust = mesh(
        plane_geo!(0.42, 0.22),
        lambert!({ map: &hole_dust_tex(), transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 }),
    );
    dust.rotation().x = -PI / 2.0;
    dust.position().set(0, -h_height + 0.002, 0.10);
    root.add(&dust);
    root
}

/// The far board's back seen through a hole: grey-brown kraft paper with
/// its dust, lit only where the hole lets the light in (or the rough grey
/// of a broken block's core).
fn hole_back_tex(masonry: bool) -> Tex {
    thread_local! { static T: std::cell::OnceCell<[Tex; 2]> = const { std::cell::OnceCell::new() }; }
    let pair = T.with(|c| {
        c.get_or_init(|| {
            let paper = canvas_tex_opt(256, 320, false, |ctx, w, h| {
                ctx.set_fill_style("#4c463d");
                ctx.fill_rect(0, 0, w, h);
                // the paper's fibre and the dust run down it
                for _ in 0..900 {
                    let (x, y) = (rnd() * w, rnd() * h);
                    ctx.set_fill_style(format!(
                        "rgba({},{},{},{})",
                        60 + (rnd() * 60.0) as i32,
                        55 + (rnd() * 50.0) as i32,
                        45 + (rnd() * 40.0) as i32,
                        0.15 + rnd() * 0.2
                    ));
                    ctx.fill_rect(x, y, 1.0 + rnd() * 2.0, 1.0 + rnd() * 5.0);
                }
                // the maker's line printed along the back paper, faint
                ctx.set_fill_style("rgba(40,52,70,0.45)");
                ctx.set_font("bold 13px Helvetica");
                ctx.set_text_align("left");
                ctx.fill_text("1/2 IN. REGULAR GYPSUM BOARD   ASTM C1396   1/2 IN. REGULAR", -30, h * 0.47);
                // gypsum dust settled where the board's crumbs fell, lower down
                let gr = ctx.create_linear_gradient(0, h * 0.55, 0, h);
                gr.add_color_stop(0, "rgba(220,214,200,0)");
                gr.add_color_stop(1, "rgba(220,214,200,0.35)");
                ctx.set_fill_style(gr);
                ctx.fill_rect(0, h * 0.55, w, h * 0.45);
                // the light: only what comes in through the hole
                let lg = ctx.create_radial_gradient(w / 2.0, h / 2.0, 6, w / 2.0, h / 2.0, w * 0.55);
                // (the bay's own shade, not its darkness: the light map darkens it as
                // it darkens the room, and an albedo under 0.008 is no paper's)
                lg.add_color_stop(0, "rgba(0,0,0,0.35)");
                lg.add_color_stop(0.22, "rgba(0,0,0,0.5)");
                lg.add_color_stop(0.6, "rgba(0,0,0,0.65)");
                lg.add_color_stop(1, "rgba(0,0,0,0.75)");
                ctx.set_fill_style(lg);
                ctx.fill_rect(0, 0, w, h);
            });
            let block = canvas_tex_opt(256, 320, false, |ctx, w, h| {
                ctx.set_fill_style("#6f6b64");
                ctx.fill_rect(0, 0, w, h);
                // the aggregate broken through
                for _ in 0..700 {
                    let (x, y) = (rnd() * w, rnd() * h);
                    let r = 1.0 + rnd() * 4.0;
                    let v = (60.0 + rnd() * 90.0) as i32;
                    ctx.set_fill_style(format!("rgba({},{},{},{})", v, v - 4, v - 10, 0.5 + rnd() * 0.4));
                    ctx.begin_path();
                    ctx.ellipse(x, y, r, r * (0.6 + rnd() * 0.4), rnd() * 3.0, 0, 7);
                    ctx.fill();
                }
                let lg = ctx.create_radial_gradient(w / 2.0, h / 2.0, 6, w / 2.0, h / 2.0, w * 0.55);
                lg.add_color_stop(0, "rgba(0,0,0,0.05)");
                lg.add_color_stop(0.4, "rgba(0,0,0,0.35)");
                lg.add_color_stop(1, "rgba(0,0,0,0.75)");
                ctx.set_fill_style(lg);
                ctx.fill_rect(0, 0, w, h);
            });
            [paper, block]
        })
        .clone()
    });
    pair[masonry as usize].clone()
}

/// Round a hole on the wall: hairline cracks out from its rim, the dent the
/// blow left, grey smudges of hands, a shoe's black scuff under the big one
/// (transparent where unmarked). `k` the hole's size: 0 a fist, 1, 2 a kick.
fn hole_decal_tex(k: usize) -> Tex {
    thread_local! { static T: std::cell::OnceCell<Vec<Tex>> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            [(0.056, 0.046), (0.076, 0.062), (0.105, 0.080)]
                .iter()
                .enumerate()
                .map(|(ki, &(rx, ry))| {
                    canvas_tex_opt(512, 512, false, move |ctx, w, h| {
                        ctx.clear_rect(0, 0, w, h);
                        let px = w / 0.56; // pixels a metre
                        let (cx, cy) = (w / 2.0, h / 2.0);
                        let (ex, ey) = (rx * px, ry * px);
                        // the dent: the board pushed in round the hole, its shade below
                        let dent = ctx.create_radial_gradient(cx, cy + ey * 0.15, ex * 0.9, cx, cy + ey * 0.15, ex * 1.9);
                        dent.add_color_stop(0, "rgba(30,24,16,0.22)");
                        dent.add_color_stop(1, "rgba(30,24,16,0)");
                        ctx.set_fill_style(dent);
                        ctx.fill_rect(0, 0, w, h);
                        // the cracks: from the rim, wandering, now and then forking
                        let n = 4 + ki * 2;
                        for j in 0..n {
                            let a0 = j as f64 / n as f64 * std::f64::consts::TAU + rnd() * 0.6;
                            let mut x = cx + a0.cos() * ex * 0.95;
                            let mut y = cy + a0.sin() * ey * 0.95;
                            let mut a = a0;
                            let len = (0.35 + rnd() * 0.9) * ex;
                            let steps = 14;
                            let mut pts = vec![(x, y)];
                            for _ in 0..steps {
                                a += (rnd() - 0.5) * 0.9;
                                let l = len / steps as f64;
                                x += a.cos() * l;
                                y += a.sin() * l * 0.9 + l * 0.08; // they run a little downward
                                pts.push((x, y));
                            }
                            // its lit lip, then the crack itself
                            for (col, lw, dx) in [("rgba(250,245,230,0.22)", 1.4, 1.0), ("rgba(28,22,16,0.6)", 1.0, 0.0)].iter() {
                                ctx.set_stroke_style(*col);
                                ctx.begin_path();
                                for (q, &(x, y)) in pts.iter().enumerate() {
                                    let fade = 1.0 - q as f64 / pts.len() as f64;
                                    ctx.set_line_width(lw * (0.35 + 0.9 * fade));
                                    if q == 0 {
                                        ctx.move_to(x + dx, y + dx);
                                    } else {
                                        ctx.line_to(x + dx, y + dx);
                                    }
                                }
                                ctx.stroke();
                            }
                            if rnd() < 0.5 {
                                let (bx, by) = pts[5 + (rnd() * 6.0) as usize];
                                let b = a0 + (rnd() - 0.5) * 2.0;
                                ctx.set_stroke_style("rgba(28,22,16,0.6)");
                                ctx.set_line_width(0.8);
                                ctx.begin_path();
                                ctx.move_to(bx, by);
                                let (mut x2, mut y2, mut a2) = (bx, by, b);
                                for _ in 0..6 {
                                    a2 += (rnd() - 0.5) * 1.0;
                                    x2 += a2.cos() * len / 22.0;
                                    y2 += a2.sin() * len / 22.0;
                                    ctx.line_to(x2, y2);
                                }
                                ctx.stroke();
                            }
                        }
                        // hands: grey smudges
                        for _ in 0..5 {
                            let a = rnd() * 6.3;
                            let r = (1.3 + rnd() * 1.4) * ex;
                            let (x, y) = (cx + a.cos() * r, cy + a.sin() * r * 0.9);
                            let s = ctx.create_radial_gradient(x, y, 0, x, y, 10.0 + rnd() * 16.0);
                            s.add_color_stop(0, "rgba(50,44,38,0.16)");
                            s.add_color_stop(1, "rgba(50,44,38,0)");
                            ctx.set_fill_style(s);
                            ctx.fill_rect(x - 30.0, y - 30.0, 60, 60);
                        }
                        // the kick's shoe, black rubber dragged down under it
                        if ki == 2 {
                            ctx.set_stroke_style("rgba(16,14,12,0.45)");
                            ctx.set_line_cap("round");
                            for k in 0..4 {
                                ctx.set_line_width(2.0 + rnd() * 3.0);
                                let x = cx - 20.0 + k as f64 * 12.0 + rnd() * 6.0;
                                ctx.begin_path();
                                ctx.move_to(x, cy + ey * 1.15);
                                ctx.bezier_curve_to(x + 4.0, cy + ey * 1.5, x - 3.0, cy + ey * 1.8, x + 2.0, cy + ey * (2.0 + rnd() * 0.6));
                                ctx.stroke();
                            }
                        }
                    })
                })
                .collect()
        })[k]
            .clone()
    })
}

/// Gypsum dust on the floor under a hole: fine white powder, heaviest at the
/// wall (the plane's top edge), with crumbs in it.
fn hole_dust_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            canvas_tex_opt(256, 128, false, |ctx, w, h| {
                ctx.clear_rect(0, 0, w, h);
                for _ in 0..40 {
                    let (x, y) = (w * (0.2 + 0.6 * rnd()), h * (0.05 + 0.55 * rnd() * rnd()));
                    let r = 8.0 + rnd() * 26.0;
                    let gr = ctx.create_radial_gradient(x, y, 0, x, y, r);
                    gr.add_color_stop(0, "rgba(226,220,206,0.22)");
                    gr.add_color_stop(1, "rgba(226,220,206,0)");
                    ctx.set_fill_style(gr);
                    ctx.fill_rect(x - r, y - r, r * 2.0, r * 2.0);
                }
                for _ in 0..160 {
                    let (x, y) = (w * (0.15 + 0.7 * rnd()), h * rnd() * rnd() * 0.9);
                    ctx.set_fill_style(format!("rgba(236,230,216,{})", 0.4 + rnd() * 0.5));
                    ctx.fill_rect(x, y, 1.0 + rnd() * 1.5, 1.0 + rnd() * 1.5);
                }
            })
        })
        .clone()
    })
}
