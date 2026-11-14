//! `place_rusted_pipes`
use game::prelude::*;
use placers_kit::on_wall::mix01;
use placers_kit::on_wall::wall_face;
use placers_kit::on_wall::wall_turn;
use placers_kit::place::WallCand;
use placers_kit::place::shuffle;
use placers_kit::place::wall_candidates;
use placers_kit::wall_plates::part;

// Rusted pipes: 0-2 runs of 2 in steel pipe per level along walls,
// low (0.15-0.35 m) (Blender family `wall_hole_pipe_run_posters_and_hooks`): malleable iron fittings, strut
// clamps, now and then a gate valve, the old paint flaking off them and rust
// weeping from the joints down the wall.
pub(crate) fn place_rusted_pipes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.60 {
        return;
    } // 40% chance
    let count = 1 + (rnd() * 2.0).floor() as usize;
    let mut candidates = wall_candidates(g);
    shuffle(&mut candidates);
    let mut i = 0;
    while i < count && i < candidates.len() {
        let c = candidates[i];
        // its run: 2.8-3.4 m along the wall, its axis 0.08 m off it
        let pipe_len = 2.8 + rnd() * 0.6;
        let pipe_height = 0.15 + rnd() * 0.20;
        // (the old bracket's 30% chance, the same draw): a gate valve on the run
        let valve = rnd() < 0.30;
        let (rot, _) = wall_turn(&c);
        let held = wall_runs_on(g, &c);
        let run = build_pipe_run(pipe_len, pipe_height, valve, held);
        run.rotation().y = rot;
        let (fx, fz) = wall_face(g, &c);
        if c.dx.abs() == 1 {
            run.position().set(fx - c.dx as f64 * 0.08, pipe_height, g.wz(c.z));
        } else {
            run.position().set(g.wx(c.x), pipe_height, fz - c.dz as f64 * 0.08);
        }
        g.world.add(&run);
        i += 1;
    }
}

/// A straight length of 2 in pipe from `a` to `b` (along one axis), its
/// paint and rust laid a metre to a repeat whatever its length.
fn pipe_length(mat: &Mat, a: Vec3, b: Vec3) -> Obj {
    let d = b - a;
    let len = d.length().max(0.001);
    let geo = cyl_geo!(0.03015, 0.03015, len, 16, 1, true);
    let uv: Vec<f32> = geo.r().uv.clone();
    let uv: Vec<f32> = uv.iter().enumerate().map(|(i, &v)| if i % 2 == 1 { v * len as f32 } else { v }).collect();
    geo.set_uv(uv);
    let m = mesh(geo, mat);
    m.position().set((a.x + b.x) / 2.0, (a.y + b.y) / 2.0, (a.z + b.z) / 2.0);
    if d.x.abs() > d.y.abs() && d.x.abs() > d.z.abs() {
        m.rotation().z = PI / 2.0;
    } else if d.z.abs() > d.y.abs() {
        m.rotation().x = PI / 2.0;
    }
    m
}

/// A fitting part at `at`, turned by `outer_y` then (inside that) by
/// `inner` (x, z).
fn pipe_fitting(name: &str, mat: &Mat, at: Vec3, outer_y: f64, inner: (f64, f64)) -> Obj {
    let o = group();
    o.position().copy(&at);
    o.rotation().y = outer_y;
    let m = mesh(part(name, 0.1, 0.1, 0.08), mat);
    m.rotation().x = inner.0;
    m.rotation().z = inner.1;
    o.add(&m);
    o
}

/// Whether the wall behind candidate `c` runs on past its tile to the run's
/// left (-x) and right (+x), as the run's group is turned.
fn wall_runs_on(g: &Game, c: &WallCand) -> [bool; 2] {
    let (rot, _) = wall_turn(c);
    let (ux, uz) = (rot.cos().round() as i32, (-rot.sin()).round() as i32);
    let (nx, nz) = (c.x + c.dx, c.z + c.dz);
    [g.is_wall_tile(nx - ux, nz - uz), g.is_wall_tile(nx + ux, nz + uz)]
}

/// The run: its group's origin on the pipe's axis at the middle of the run,
/// x along the wall, y up, z out of the wall (the wall at z = -0.08, the
/// floor at y = -h). Each end turns into the wall, down through the floor,
/// or up the wall and into it at 1.0-1.6 m; a coupling, a union, two strut
/// clamps, a gate valve when `valve`; rust weeping from the joints.
fn build_pipe_run(len: f64, h: f64, valve: bool, held: [bool; 2]) -> Obj {
    const EA: f64 = 0.057; // centre to end of a 2 in elbow
    const IN: f64 = 0.017; // how far a pipe's thread goes into its fitting
    const WALL: f64 = -0.08;
    let run = group();
    let k1 = mix01(len, h);
    let k2 = mix01(h * 1.7, len * 0.3);
    let paint = ((k1 * 7.0).fract() * 3.0) as usize;
    let pipe_mat = lambert!({ map: &pipe_tex(paint, false) });
    let fit_mat = lambert!({ map: &pipe_tex(paint, true) });
    let galv = lambert!({ map: &pipe_tex(3, true), color: 0xffffff });
    let plate = phong!({ color: 0x77766f, specular: 0x262626, shininess: 26 });
    let xe = len / 2.0;
    // the run itself, through its inline fittings
    run.add(&pipe_length(&pipe_mat, v3(-xe + EA - IN, 0, 0), v3(xe - EA + IN, 0, 0)));
    // where the wall runs on past an end, the pipe may go into it or up it;
    // where it stops (a pillar, a wall's end) the pipe goes down into the floor
    let ends = [if held[0] { (k1 * 3.0) as usize % 3 } else { 1 }, if held[1] { (k2 * 3.0) as usize % 3 } else { 1 }];
    let rise = 1.0 + (k1 * 13.0).fract() * 0.6 - h; // the top of a riser, above the run
    let mut joints: Vec<f64> = vec![];
    for (side, &end) in ends.iter().enumerate() {
        let sx = if side == 0 { -1.0 } else { 1.0 };
        let x = sx * xe;
        let oy = if side == 0 { 0.0 } else { PI };
        joints.push(x);
        match end {
            0 => {
                // into the wall: the elbow's other leg back (its legs are +x and +y as
                // modelled; turned about x its +y leg goes to -z, the wall, on either
                // side once the outer turn is applied), a nipple, its collar
                run.add(&pipe_fitting("Real__pipeElbow", &fit_mat, v3(x, 0, 0), oy, (sx * PI / 2.0, 0.0)));
                run.add(&pipe_length(&pipe_mat, v3(x, 0, -EA + IN), v3(x, 0, WALL)));
                let col = mesh(part("Real__pipeCollar", 0.115, 0.115, 0.012), &plate);
                col.position().set(x, 0, WALL);
                run.add(&col);
            }
            1 => {
                // down through the floor, its collar on the floor
                run.add(&pipe_fitting("Real__pipeElbow", &fit_mat, v3(x, 0, 0), oy, (PI, 0.0)));
                run.add(&pipe_length(&pipe_mat, v3(x, -EA + IN, 0), v3(x, -h, 0)));
                let col = mesh(part("Real__pipeCollar", 0.115, 0.115, 0.012), &plate);
                col.rotation().x = -PI / 2.0;
                col.position().set(x, -h, 0);
                run.add(&col);
            }
            _ => {
                // up the wall on a clamp, then into it
                run.add(&pipe_fitting("Real__pipeElbow", &fit_mat, v3(x, 0, 0), oy, (0.0, 0.0)));
                run.add(&pipe_length(&pipe_mat, v3(x, EA - IN, 0), v3(x, rise - EA + IN, 0)));
                run.add(&pipe_fitting("Real__pipeElbow", &fit_mat, v3(x, rise, 0), PI / 2.0, (0.0, -PI / 2.0)));
                run.add(&pipe_length(&pipe_mat, v3(x, rise, -EA + IN), v3(x, rise, WALL)));
                let col = mesh(part("Real__pipeCollar", 0.115, 0.115, 0.012), &plate);
                col.position().set(x, rise, WALL);
                run.add(&col);
                let cl = mesh(part("Real__pipeClamp", 0.04, 0.16, 0.12), &galv);
                cl.rotation().z = PI / 2.0;
                cl.position().set(x, rise * 0.55, 0);
                run.add(&cl);
            }
        }
    }
    // two strut clamps on the run
    for (side, sx) in [-1.0f64, 1.0].iter().enumerate() {
        let cl = mesh(part("Real__pipeClamp", 0.04, 0.16, 0.12), &galv);
        cl.position().x = sx * if held[side] { xe - 0.45 } else { 0.3 };
        run.add(&cl);
    }
    // a coupling where two lengths were joined, a union by the valve's place
    let xc = -xe + 0.9 + (k2 * 5.0).fract() * 0.5;
    run.add(&pipe_fitting("Real__pipeCoupling", &fit_mat, v3(xc, 0, 0), 0.0, (0.0, 0.0)));
    joints.push(xc);
    // now and then a tee, its branch back into the wall
    if (k1 * 17.0).fract() < 0.6 {
        let xt = xc + 0.55;
        run.add(&pipe_fitting("Real__pipeTee", &fit_mat, v3(xt, 0, 0), 0.0, (-PI / 2.0, 0.0)));
        run.add(&pipe_length(&pipe_mat, v3(xt, 0, -EA + IN), v3(xt, 0, WALL)));
        let col = mesh(part("Real__pipeCollar", 0.115, 0.115, 0.012), &plate);
        col.position().set(xt, 0, WALL);
        run.add(&col);
        joints.push(xt);
    }
    let xv = xe - 0.85;
    if valve {
        let bronze = lambert!({ color: 0x5e4c32 });
        run.add(&pipe_fitting("Real__pipeValve", &bronze, v3(xv, 0, 0), 0.0, (0.0, 0.0)));
        let wheel = mesh(part("Real__pipeWheel", 0.115, 0.03, 0.115), lambert!({ color: 0x5a2219 }));
        wheel.position().set(xv, 0.150, 0);
        wheel.rotation().y = k2 * 1.5;
        run.add(&wheel);
        joints.push(xv);
        run.add(&pipe_fitting("Real__pipeUnion", &fit_mat, v3(xv - 0.16, 0, 0), 0.0, (0.0, 0.0)));
    } else {
        run.add(&pipe_fitting("Real__pipeUnion", &fit_mat, v3(xv, 0, 0), 0.0, (0.0, 0.0)));
        joints.push(xv);
    }
    // the rust that wept from the joints: down the wall, and dried on the floor
    let weep = lambert!({ map: &rust_weep_tex(), transparent: true, depthWrite: false });
    for (j, &x) in joints.iter().enumerate() {
        if (k1 * 31.0 + j as f64 * 0.37).fract() < 0.25 {
            continue; // not every joint leaks
        }
        let d = mesh(plane_geo!(0.12, 0.40), &weep);
        d.position().set(x + 0.01, -0.21, WALL + 0.0008);
        run.add(&d);
    }
    let stain = mesh(
        plane_geo!(0.34, 0.24),
        lambert!({ map: &rust_floor_tex(), transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 }),
    );
    stain.rotation().x = -PI / 2.0;
    stain.position().set(joints[(k2 * 9.0) as usize % joints.len()], -h + 0.002, WALL + 0.12);
    run.add(&stain);
    run
}

/// A pipe's old paint over its steel (`paint`: 0 grey, 1 green, 2 cream),
/// flaked off in patches, rust in them and blooming round them; a fitting's
/// (`fitting`) rustier still. 128 px round, a metre along.
fn pipe_tex(paint: usize, fitting: bool) -> Tex {
    thread_local! { static T: std::cell::OnceCell<Vec<Tex>> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            let mut v = vec![];
            for fit in [false, true] {
                for (base, edge) in [("#5f625e", "#4c4f4b"), ("#56614f", "#454f40"), ("#9a917a", "#7f7864"), ("#7a7c79", "#5d5f5c")] {
                    v.push(canvas_tex(128, 512, move |ctx, w, h| {
                        ctx.set_fill_style(base);
                        ctx.fill_rect(0, 0, w, h);
                        // brush marks and grime in the paint
                        for _ in 0..140 {
                            ctx.set_fill_style(format!("rgba(30,26,20,{})", 0.04 + rnd() * 0.08));
                            ctx.fill_rect(rnd() * w, rnd() * h, 1.0 + rnd() * 3.0, 6.0 + rnd() * 30.0);
                        }
                        // the flakes: paint lifted off, the steel rusted beneath
                        let n = if fit { 120 } else { 85 };
                        for _ in 0..n {
                            let (x, y) = (rnd() * w, rnd() * h);
                            let r = 2.0 + rnd() * if fit { 14.0 } else { 12.0 };
                            // the rust bloom round it
                            let gr = ctx.create_radial_gradient(x, y, r * 0.4, x, y, r * 2.2);
                            gr.add_color_stop(0, "rgba(110,56,24,0.55)");
                            gr.add_color_stop(1, "rgba(110,56,24,0)");
                            ctx.set_fill_style(gr);
                            ctx.fill_rect(x - r * 2.2, y - r * 2.2, r * 4.4, r * 4.4);
                            // the flake's edge, then the bare rust inside it
                            ctx.set_fill_style(edge);
                            ctx.begin_path();
                            let k = 7;
                            for q in 0..k {
                                let a = q as f64 / k as f64 * 6.283;
                                let rr = r * (0.7 + rnd() * 0.5) + 1.0;
                                if q == 0 {
                                    ctx.move_to(x + a.cos() * rr, y + a.sin() * rr);
                                } else {
                                    ctx.line_to(x + a.cos() * rr, y + a.sin() * rr);
                                }
                            }
                            ctx.close_path();
                            ctx.fill();
                            ctx.set_fill_style(if rnd() < 0.5 { "#5a301a" } else { "#3e2618" });
                            ctx.begin_path();
                            for q in 0..k {
                                let a = q as f64 / k as f64 * 6.283;
                                let rr = r * (0.55 + rnd() * 0.45);
                                if q == 0 {
                                    ctx.move_to(x + a.cos() * rr, y + a.sin() * rr);
                                } else {
                                    ctx.line_to(x + a.cos() * rr, y + a.sin() * rr);
                                }
                            }
                            ctx.close_path();
                            ctx.fill();
                        }
                        // rust run along the lower side (a quarter of the way round)
                        let gr = ctx.create_linear_gradient(w * 0.62, 0, w * 0.88, 0);
                        gr.add_color_stop(0, "rgba(96,50,22,0)");
                        gr.add_color_stop(0.5, if fit { "rgba(96,50,22,0.55)" } else { "rgba(96,50,22,0.3)" });
                        gr.add_color_stop(1, "rgba(96,50,22,0)");
                        ctx.set_fill_style(gr);
                        ctx.fill_rect(0, 0, w, h);
                        // dust on what faces up
                        let gd = ctx.create_linear_gradient(w * 0.1, 0, w * 0.4, 0);
                        gd.add_color_stop(0, "rgba(150,140,120,0)");
                        gd.add_color_stop(0.5, "rgba(150,140,120,0.18)");
                        gd.add_color_stop(1, "rgba(150,140,120,0)");
                        ctx.set_fill_style(gd);
                        ctx.fill_rect(0, 0, w, h);
                    }));
                }
            }
            v
        })[paint % 4 + if fitting { 4 } else { 0 }]
        .clone()
    })
}

/// Rust wept from a joint down a wall: thin runs from the top, wider and
/// paler as they spread, a tide line where each dried (transparent elsewhere).
fn rust_weep_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            canvas_tex_opt(96, 320, false, |ctx, w, h| {
                ctx.clear_rect(0, 0, w, h);
                for k in 0..7 {
                    let x0 = w * (0.3 + 0.4 * rnd());
                    let len = h * (0.25 + 0.7 * rnd());
                    let a = 0.22 + 0.30 * rnd();
                    let mut x = x0;
                    let steps = 40;
                    for q in 0..steps {
                        let t = q as f64 / steps as f64;
                        let y = t * len;
                        x += (rnd() - 0.5) * 1.6;
                        let wd = 1.5 + t * (4.0 + k as f64 * 0.6);
                        ctx.set_fill_style(format!("rgba(104,52,22,{})", a * (1.0 - t * 0.7)));
                        ctx.fill_rect(x - wd / 2.0, y, wd, len / steps as f64 + 1.0);
                    }
                    // the tide line where it stopped
                    ctx.set_fill_style(format!("rgba(70,34,14,{})", a * 0.9));
                    ctx.begin_path();
                    ctx.ellipse(x, len, 3.0 + rnd() * 3.0, 2.0, 0, 0, 7);
                    ctx.fill();
                }
                let gr = ctx.create_linear_gradient(0, 0, 0, 30);
                gr.add_color_stop(0, "rgba(84,42,18,0.5)");
                gr.add_color_stop(1, "rgba(84,42,18,0)");
                ctx.set_fill_style(gr);
                ctx.fill_rect(w * 0.2, 0, w * 0.6, 30);
            })
        })
        .clone()
    })
}

/// Where the drips dried on the floor: rings of rust, darker at their rims.
fn rust_floor_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            canvas_tex_opt(192, 128, false, |ctx, w, h| {
                ctx.clear_rect(0, 0, w, h);
                for _ in 0..9 {
                    let (x, y) = (w * (0.3 + 0.4 * rnd()), h * (0.15 + 0.5 * rnd()));
                    let r = 8.0 + rnd() * 26.0;
                    let gr = ctx.create_radial_gradient(x, y, r * 0.2, x, y, r);
                    gr.add_color_stop(0, "rgba(98,52,24,0.20)");
                    gr.add_color_stop(0.85, "rgba(90,46,20,0.32)");
                    gr.add_color_stop(1, "rgba(90,46,20,0)");
                    ctx.set_fill_style(gr);
                    ctx.begin_path();
                    ctx.ellipse(x, y, r, r * (0.6 + 0.3 * rnd()), rnd() * 3.0, 0, 7);
                    ctx.fill();
                }
            })
        })
        .clone()
    })
}
