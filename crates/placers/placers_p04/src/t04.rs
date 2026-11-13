//! Batch `t04`'s props in this crate, made real (`docs/design/PROPS.md` P3): the
//! soap-smudged washroom mirror and the frosted window; their placers and
//! what only they use (`assets/src/blender/props/t04.py`, `t04.notes.md`).
use super::*;
use game::assets::props::prop_part_as_modelled;

/// The Blender part `name`, as modelled, in `grp` with `mat`
fn t04_put(grp: &Obj, name: &str, mat: &Mat) -> Option<Obj> {
    let m = mesh(prop_part_as_modelled(name)?, mat);
    grp.add(&m);
    Some(m)
}

/// Something of where a thing was put, to vary what its painters paint
/// (a painter's own stream starts the same on every floor)
fn t04_vary(x: f64, z: f64, k: f64) -> u32 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for v in [x, z, k] {
        h ^= v.to_bits();
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    (h ^ (h >> 29) ^ (h >> 47)) as u32
}

// ---- painting kit (per pixel, pure: safe for `par_rows`) ----
fn t04_hash(seed: u32, i: i32, j: i32) -> f64 {
    let mut h = seed ^ (i as u32).wrapping_mul(0x27d4_eb2d) ^ (j as u32).wrapping_mul(0x1656_67b1);
    h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
    h = (h ^ (h >> 13)).wrapping_mul(0xc2b2_ae35);
    h ^= h >> 16;
    f64::from(h) / 4_294_967_296.0
}

fn t04_vn(seed: u32, x: f64, y: f64) -> f64 {
    let (xf, yf) = (x.floor(), y.floor());
    let (i, j) = (xf as i32, yf as i32);
    let (u, v) = (x - xf, y - yf);
    let (u, v) = (u * u * (3.0 - 2.0 * u), v * v * (3.0 - 2.0 * v));
    let (a, b) = (t04_hash(seed, i, j), t04_hash(seed, i + 1, j));
    let (c, d) = (t04_hash(seed, i, j + 1), t04_hash(seed, i + 1, j + 1));
    a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v
}

fn t04_fbm(seed: u32, x: f64, y: f64, oct: u32) -> f64 {
    let (mut s, mut amp, mut f, mut n) = (0.0, 0.5, 1.0, 0.0);
    for k in 0..oct {
        s += amp * t04_vn(seed.wrapping_add(k * 7919), x * f, y * f);
        n += amp;
        amp *= 0.5;
        f *= 2.03;
    }
    s / n
}

fn t04_ss(e0: f64, e1: f64, x: f64) -> f64 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Fills the canvas pixel by pixel on every core: `f(u, v)` (u right, v up,
/// both 0..1, as the texture is laid) gives straight sRGB and alpha, 0..1
fn t04_fill(ctx: &mut Canvas, w: f64, h: f64, f: impl Fn(f64, f64) -> [f64; 4] + Sync) {
    let iw = w as usize;
    let mut img = ctx.create_image_data(w, h);
    par_rows(&mut img.data, iw * 4, |j, out| {
        for i in 0..iw {
            let c = f((i as f64 + 0.5) / w, 1.0 - (j as f64 + 0.5) / h);
            for k in 0..4 {
                out[i * 4 + k] = (c[k].clamp(0.0, 1.0) * 255.0).round() as u8;
            }
        }
    });
    ctx.put_image_data(&img, 0, 0);
}

/// A soft blob, squashed whole (`k`) and turned, so its fade is never cut
fn t04_blob(ctx: &mut Canvas, x: f64, y: f64, r: f64, k: f64, turn: f64, stops: &[(f64, String)]) {
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

/// A pane of glass (`Patch::Glass`): `room` how bright the room reads in
/// it, `back` the light behind it (linear), `milk` how much of that light
/// its frosting scatters, `fresnel` its reflection square on (0.86 a
/// silvered one); its dirt from the map's grey, in `dirt`'s colour
fn t04_glass_mat(map: Tex, name: &str, room: f32, back: [f32; 3], milk: f32, fresnel: f32, dirt: [f32; 4], shine: f64) -> Mat {
    let m = phong!({ color: 0xffffff, specular: 0x2a2a2a, shininess: shine, transparent: true, depthWrite: false });
    {
        let mut mm = m.m();
        mm.map = Some(map);
        mm.patch = Patch::Glass;
        mm.look_settled = true;
        mm.name = name.into();
    }
    {
        let pu = m.r().patch_uniforms.clone();
        let f = &mut pu.borrow_mut().f;
        f[0] = room;
        f[1] = back[0];
        f[2] = back[1];
        f[3] = back[2];
        // no breath's mist
        f[4] = 0.5;
        f[5] = 0.6;
        f[6] = 0.3;
        f[7] = 0.0;
        f[8] = dirt[0];
        f[9] = dirt[1];
        f[10] = dirt[2];
        f[11] = dirt[3];
        f[12] = milk;
        f[13] = fresnel;
    }
    m
}

// ===================================================== the smudged mirror

/// The mirror's glass as seen inside its frame (w, h), its face's distance
/// from the wall (`Real__smudgeMirrorFrame`'s hem, t04.py SM_*), and how much
/// more of it runs under the frame's lip (both sides)
const SM_GLASS: [f64; 3] = [0.380, 0.482, 0.0172];
const SM_UNDER: f64 = 0.006;

// Soap-smudged wall mirror: 4% chance per level. A 16 x 20 in
// stainless channel-frame washroom mirror (Blender batch `t04`, the Bradley
// 781 class) on its hanger: silvered glass that gives back the room's light
// (`Patch::Glass`), the silver just starting to go at its foot; on its face
// the last cleaning's smears, wiped in arcs with a soapy cloth and left to
// dry in streaks, the splashes from the basin under it, fingers' smudges at
// its edges, flecks of toothpaste, dust over its top.
pub(super) fn place_smudge_mirrors(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    }
    let Some((x, z, dx, dz)) = pick_wall(g) else { return };
    let mirror = group();
    let mount_y = 1.55;
    wall_mount(g, &mirror, x, z, dx, dz, 0.005);
    let p = mirror.pos();
    let vary = t04_vary(p.x, p.z, mount_y);
    // the parts are modelled with the wall at z = 0: 5 mm behind the spot
    let body = group();
    body.position().set(0, mount_y, -0.005);
    mirror.add(&body);
    t04_put(&body, "Real__smudgeMirrorFrame", &phong!({ color: 0x55585a, specular: 0x8a8d90, shininess: 48 }));
    // the black paint on the glass's back, seen where the silver has gone
    let back = mesh(plane_geo!(SM_GLASS[0] + 0.01, SM_GLASS[1] + 0.01), lambert!({ color: 0x141312 }));
    back.position().z = SM_GLASS[2] - 0.0062;
    body.add(&back);
    let glass = mesh(
        plane_geo!(SM_GLASS[0] + SM_UNDER, SM_GLASS[1] + SM_UNDER),
        // (the lamp in it a small hot spot, not a disc of glare: `MIRROR_SHINE`)
        t04_glass_mat(sm_silver_tex(vary), "mirror_glass", 0.13, [0.0; 3], 0.0, 0.86, [1.0, 0.016, 0.015, 0.013], MIRROR_SHINE),
    );
    glass.position().z = SM_GLASS[2] - 0.0001;
    glass.set_render_order(2);
    body.add(&glass);
    let film = mesh(
        plane_geo!(SM_GLASS[0], SM_GLASS[1]),
        lambert!({ map: &sm_film_tex(vary), transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 }),
    );
    film.position().z = SM_GLASS[2] + 0.0003;
    film.set_render_order(3);
    body.add(&film);
    g.world.add(&mirror);
}

/// Where the silver has gone (grey: how much, to the black paint behind):
/// only begun, a feathered black edge in stretches along its foot and in its
/// lower corners, a few spots ahead of it, pinholes
fn sm_silver_tex(vary: u32) -> Tex {
    canvas_tex_opt(192, 240, false, move |ctx, w, h| {
        let seed = vary ^ 0x5117;
        let (gw, gh) = (SM_GLASS[0] + SM_UNDER, SM_GLASS[1] + SM_UNDER);
        t04_fill(ctx, w, h, |u, v| {
            let (x, y) = ((u - 0.5) * gw, (v - 0.5) * gh);
            // distance in from the nearest edge, the foot's counted double
            let dx = gw / 2.0 - x.abs();
            let dfoot = y + gh / 2.0;
            let wave = t04_fbm(seed, x * 18.0, y * 18.0, 3);
            let deep = 0.004 + 0.010 * t04_ss(0.45, 0.75, t04_fbm(seed ^ 0x33, x * 6.0, 0.5, 3));
            let foot = 1.0 - t04_ss(deep * 0.5, deep * (1.2 + wave), dfoot);
            let corner = 1.0 - t04_ss(0.0, 0.03, (dx * dx + dfoot * dfoot).sqrt() - 0.012);
            let side = (1.0 - t04_ss(0.0015, 0.004 + 0.004 * wave, dx)) * (1.0 - t04_ss(0.05, 0.2, dfoot));
            let spot = if t04_hash(seed ^ 9, (u * 96.0) as i32, (v * 120.0) as i32) < 0.006 * (1.0 - t04_ss(0.0, 0.12, dfoot)) + 0.0006 {
                0.8
            } else {
                0.0
            };
            let g0 = foot.max(corner * 0.9).max(side).max(spot);
            [g0, g0, g0, 1.0]
        });
    })
}

/// The mirror's face (transparent where clean): a soapy cloth's arcs dried
/// in milky streaks, the run of its last wipe, a haze over the lower half
/// with the basin's splashes dried on it, toothpaste flecks, fingers'
/// smudges at its sides, dust settled along the top, the frame's lip seen
/// again in it
fn sm_film_tex(vary: u32) -> Tex {
    canvas_tex_opt(384, 488, false, move |ctx, w, h| {
        let mut r = SplitMix64::new(vary, 0x5aa9_e1f1);
        ctx.clear_rect(0, 0, w, h);
        // the frame's lip seen again in the glass, the glass's edge a dark line
        ctx.set_fill_style("rgba(150,152,150,0.42)");
        for (x, y, rw, rh) in [(0.0, 0.0, w, 3.0), (0.0, h - 3.0, w, 3.0), (0.0, 0.0, 3.0, h), (w - 3.0, 0.0, 3.0, h)] {
            ctx.fill_rect(x, y, rw, rh);
        }
        ctx.set_stroke_style("rgba(18,18,17,0.45)");
        ctx.set_line_width(1.2);
        ctx.stroke_rect(3.6, 3.6, w - 7.2, h - 7.2);
        // a haze of dried soap and water over the lower half
        let gr = ctx.create_linear_gradient(0, h * 0.35, 0, h);
        gr.add_color_stop(0, "rgba(200,198,190,0)");
        gr.add_color_stop(1, "rgba(200,198,190,0.16)");
        ctx.set_fill_style(&gr);
        ctx.fill_rect(0, h * 0.35, w, h * 0.65);
        // the cloth's arcs: overlapping sweeps, each a band of soapy smear
        // and streaks dragged through it, wandering, fading in and out along
        // the stroke, broken where the cloth lifted
        ctx.set_line_cap("round");
        ctx.set_line_join("round");
        let sweeps = 5 + (r.next() * 3.0) as usize;
        for _ in 0..sweeps {
            let cx = w * r.range(0.0, 1.0);
            let cy = h * r.range(0.35, 1.1);
            let rad = w * r.range(0.3, 0.6);
            let a0 = r.range(0.0, 6.3);
            let span = r.range(0.6, 1.3);
            let band = r.range(30.0, 70.0);
            let alpha = r.range(0.04, 0.08);
            let (ph1, ph2) = (r.range(0.0, 6.3), r.range(0.0, 6.3));
            let strands = 7 + (r.next() * 6.0) as usize;
            for k in 0..strands {
                let off = band * (r.next() - 0.5);
                let broad = k < 2;
                let lw = if broad { r.range(10.0, 22.0) } else { r.range(0.6, 2.0) };
                let a_k = alpha * if broad { 0.55 } else { r.range(0.4, 1.0) };
                let (t_in, t_out) = (r.range(0.0, 0.25), r.range(0.75, 1.0));
                let gap = r.range(0.2, 0.9);
                let segs = 28;
                ctx.set_line_width(lw);
                let pt = |t: f64| {
                    let a = a0 + span * t;
                    let rr = rad + off + 4.0 * (t * 9.0 + ph1).sin() + 2.5 * (t * 23.0 + ph2 + off * 0.1).sin();
                    (cx + a.cos() * rr, cy + a.sin() * rr)
                };
                for sg in 0..segs {
                    let (t0, t1) = (sg as f64 / segs as f64, (sg + 1) as f64 / segs as f64);
                    if t1 < t_in || t0 > t_out || (!broad && (t0 - gap).abs() < 0.04) {
                        continue;
                    }
                    let env = (PI * ((t0 + t1) / 2.0 - t_in) / (t_out - t_in)).sin().max(0.0);
                    ctx.set_stroke_style(format!("rgba(216,214,206,{:.3})", a_k * env));
                    let (x0, y0) = pt(t0);
                    let (x1, y1) = pt(t1);
                    ctx.begin_path();
                    ctx.move_to(x0, y0);
                    ctx.line_to(x1, y1);
                    ctx.stroke();
                }
            }
        }
        // the last wipe's run: the soapy water gathered at its foot and ran
        // a little way, wandering, beading where it stopped
        for _ in 0..(4 + (r.next() * 5.0) as usize) {
            let x = w * r.range(0.08, 0.92);
            let y0 = h * r.range(0.45, 0.8);
            let len = h * r.range(0.03, 0.14);
            ctx.set_stroke_style(format!("rgba(206,203,194,{:.3})", r.range(0.06, 0.14)));
            ctx.set_line_width(r.range(0.8, 1.6));
            ctx.begin_path();
            ctx.move_to(x, y0);
            let mut px = x;
            for k in 1..=8 {
                px += r.range(-0.8, 0.8);
                ctx.line_to(px, y0 + len * k as f64 / 8.0);
            }
            ctx.stroke();
            t04_blob(ctx, px, y0 + len, 2.2, 1.3, 0.0, &[(0.0, "rgba(212,209,200,0.28)".into()), (1.0, "rgba(212,209,200,0)".into())]);
        }
        // the basin's splashes, dried, thickest low down
        for _ in 0..180 {
            let x = r.next() * w;
            let y = h * (0.4 + 0.6 * r.next().powf(0.55));
            let rr = 0.5 + r.next() * r.next() * 2.8;
            let a = 0.08 + 0.2 * r.next();
            ctx.set_fill_style(format!("rgba(208,205,196,{a:.3})"));
            ctx.begin_path();
            ctx.ellipse(x, y, rr, rr * (0.7 + 0.3 * r.next()), r.next() * 3.0, 0, 7);
            ctx.fill();
            // a ring where it dried
            if r.next() < 0.25 {
                ctx.set_stroke_style(format!("rgba(190,186,176,{:.3})", a * 0.8));
                ctx.set_line_width(0.6);
                ctx.begin_path();
                ctx.arc(x, y, rr + 0.8, 0, 7);
                ctx.stroke();
            }
        }
        // toothpaste: white flecks at a mouth's height over the basin
        for _ in 0..(10 + (r.next() * 16.0) as usize) {
            let x = w * (0.5 + r.range(-0.3, 0.3));
            let y = h * r.range(0.62, 0.92);
            ctx.set_fill_style(format!("rgba(232,234,230,{:.2})", r.range(0.5, 0.85)));
            ctx.begin_path();
            ctx.ellipse(x, y, r.range(0.8, 2.0), r.range(0.6, 1.4), r.range(0.0, 3.0), 0, 7);
            ctx.fill();
        }
        // greasy smudges at its sides where hands set it straight, soft
        for side in [0.06, 0.94] {
            if r.next() < 0.4 {
                continue;
            }
            let (x, y) = (w * (side + r.range(-0.03, 0.03)), h * r.range(0.3, 0.7));
            let turn = r.range(-0.5, 0.5);
            for _ in 0..3 {
                let (dx, dy) = (r.range(-8.0, 8.0), r.range(-18.0, 18.0));
                t04_blob(
                    ctx,
                    x + dx,
                    y + dy,
                    r.range(6.0, 11.0),
                    r.range(0.45, 0.8),
                    turn + r.range(-0.3, 0.3),
                    &[(0.0, format!("rgba(190,186,176,{:.3})", r.range(0.08, 0.16))), (1.0, "rgba(190,186,176,0)".into())],
                );
            }
        }
        // dust along the top, and more of it in the corners
        let gt = ctx.create_linear_gradient(0, 0, 0, h * 0.22);
        gt.add_color_stop(0, "rgba(176,172,162,0.20)");
        gt.add_color_stop(1, "rgba(176,172,162,0)");
        ctx.set_fill_style(&gt);
        ctx.fill_rect(0, 0, w, h * 0.22);
        for _ in 0..10 {
            let (x, y, rr) = (r.next() * w, r.next() * h, w * r.range(0.08, 0.25));
            t04_blob(
                ctx,
                x,
                y,
                rr,
                r.range(0.5, 1.0),
                r.range(0.0, 3.0),
                &[(0.0, format!("rgba(176,172,162,{:.3})", r.range(0.03, 0.07))), (1.0, "rgba(176,172,162,0)".into())],
            );
        }
    })
}

// ===================================================== the frosted window

/// The sash's four panes (x0, x1, y0, y1), the glass's distance from the
/// wall, how much of each runs under its bead (`t04.py` FW_*)
const FW_PANES: [[f64; 4]; 4] =
    [[-0.228, -0.010, -0.146, 0.006], [0.010, 0.228, -0.146, 0.006], [-0.228, -0.010, 0.026, 0.178], [0.010, 0.228, 0.026, 0.178]];
const FW_GLASS_Z: f64 = 0.0055;
const FW_UNDER: f64 = 0.004;

// Frosted-glass window: 3% chance per level. A small side-hung casement
// of four lights in the wall (Blender batch `t04`): painted softwood, its
// architrave, window board and apron, the sash with its glazing bars and
// beads, a cockspur fastener; the glass acid-etched (frosted) so nothing
// shows through it but the light behind (`Patch::Glass`, milky). The paint
// gone yellow and chipped to the wood, dust on the board and the bars, a
// web in a pane's corner. The old 50%: a lamp still lit in the room beyond,
// the panes glowing warm with it (and its light, as before).
pub(super) fn place_frosted_windows(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    }
    let Some((x, z, dx, dz)) = pick_wall(g) else { return };
    let window_ = group();
    let mount_y = 1.70;
    // 50% chance: a lamp lit in the room behind it
    let lit = rnd() < 0.5;
    wall_mount(g, &window_, x, z, dx, dz, 0.006);
    let p = window_.pos();
    let vary = t04_vary(p.x, p.z, mount_y);
    // the parts are modelled with the wall at z = 0: 6 mm behind the spot
    let body = group();
    body.position().set(0, mount_y, -0.006);
    window_.add(&body);
    t04_put(&body, "Real__frostedWindowFrame", &lambert!({ map: &fw_paint_tex(vary) }));
    t04_put(&body, "Real__frostedWindowIron", &phong!({ color: 0x1e1d1b, specular: 0x262524, shininess: 18 }));
    // behind the glass: the lamp's warm light, or a dark room
    let back = if lit { [0.30, 0.21, 0.11] } else { [0.012, 0.012, 0.013] };
    for (k, pane) in FW_PANES.iter().enumerate() {
        let (w, h) = (pane[1] - pane[0] + FW_UNDER, pane[3] - pane[2] + FW_UNDER);
        let m = t04_glass_mat(fw_dirt_tex(vary, k), "frosted_glass", 2.4, back, 0.9, 0.04, [0.8, 0.16, 0.15, 0.13], 30.0);
        if lit {
            // the lamp behind: brighter toward the middle of the window
            let pu = m.r().patch_uniforms.clone();
            let f = &mut pu.borrow_mut().f;
            f[4] = if k % 2 == 0 { 0.95 } else { 0.05 };
            f[5] = if k < 2 { 0.95 } else { 0.05 };
            f[6] = 1.1;
            f[7] = 0.55;
        }
        let glass = mesh(plane_geo!(w, h), &m);
        glass.position().set((pane[0] + pane[1]) / 2.0, (pane[2] + pane[3]) / 2.0, FW_GLASS_Z);
        glass.set_render_order(2);
        body.add(&glass);
    }
    if lit {
        let glow = point_light!(0xffe4a0, 0.20, 1.5);
        glow.position().set(0, mount_y, -0.05);
        window_.add(&glow);
    }
    g.world.add(&window_);
}

/// The window's paint (the frame part's box faces: its front laid across
/// u, v): gloss white gone the yellow-grey of old paint, chipped at the
/// arrises to the grey wood, dirt in the corners and along the board, dust
/// on what faces up
fn fw_paint_tex(vary: u32) -> Tex {
    canvas_tex_opt(256, 216, false, move |ctx, w, h| {
        let seed = vary ^ 0xf0a1;
        t04_fill(ctx, w, h, |u, v| {
            let n = t04_fbm(seed, u * 20.0, v * 17.0, 4);
            let fine = t04_fbm(seed ^ 0x21, u * 90.0, v * 76.0, 2);
            let base = [0.74 + 0.04 * n, 0.71 + 0.04 * n, 0.62 + 0.03 * n];
            // chips: where the paint let go (most on the board and the foot
            // of the frame, where hands and things knocked it), to the grey
            // wood or the old cream undercoat
            let low = 1.0 - t04_ss(0.05, 0.45, v);
            let chip = t04_ss(0.79, 0.82, t04_fbm(seed ^ 0x77, u * 34.0, v * 29.0, 3) + 0.08 * fine + 0.07 * low);
            let wood = [0.40 + 0.05 * fine, 0.36 + 0.04 * fine, 0.30 + 0.03 * fine];
            let undercoat = [0.62, 0.58, 0.48];
            let under = if t04_fbm(seed ^ 0x13, u * 9.0, v * 9.0, 2) > 0.45 { wood } else { undercoat };
            let mut c = [0.0; 3];
            for k in 0..3 {
                c[k] = base[k] + (under[k] - base[k]) * chip;
            }
            // grime toward the foot (the board) and the corners
            let low = 1.0 - t04_ss(0.0, 0.3, v);
            // the old gloss crazed in a fine net of cracks
            let craze = (1.0 - (t04_vn(seed ^ 0x2b, u * 160.0, v * 135.0) - 0.5).abs() * 14.0).max(0.0) * 0.12;
            for k in 0..3 {
                c[k] *= 1.0 - craze;
            }
            let corner = (1.0 - t04_ss(0.0, 0.12, u.min(1.0 - u))) * 0.5;
            let dirt = (low * 0.35 + corner * 0.3) * (0.6 + 0.6 * n);
            for k in 0..3 {
                c[k] *= 1.0 - dirt * 0.45;
            }
            // flecks of dirt and fly specks
            if t04_hash(seed ^ 0x5, (u * 256.0) as i32, (v * 216.0) as i32) < 0.003 {
                for k in &mut c {
                    *k *= 0.7;
                }
            }
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// A frosted pane's dirt (grey: how much): dust settled along its foot and
/// in its corners, fly specks, a web tangled into one corner of one pane
fn fw_dirt_tex(vary: u32, k: usize) -> Tex {
    canvas_tex_opt(96, 72, false, move |ctx, w, h| {
        let seed = vary ^ (0x91 + k as u32 * 0x3d);
        let web = t04_hash(seed, 7, k as i32) < 0.3;
        let wc = (t04_hash(seed, 1, 2) < 0.5, t04_hash(seed, 2, 1) < 0.5);
        t04_fill(ctx, w, h, |u, v| {
            let n = t04_fbm(seed, u * 8.0, v * 6.0, 3);
            let foot = (1.0 - t04_ss(0.0, 0.25, v)) * 0.5;
            let du = u.min(1.0 - u);
            let dv = v.min(1.0 - v);
            let corner = (1.0 - t04_ss(0.0, 0.18, (du * du + dv * dv).sqrt())) * 0.5;
            let mut d = (0.12 + 0.18 * n + foot + corner).min(1.0);
            if t04_hash(seed ^ 0x44, (u * 96.0) as i32, (v * 72.0) as i32) < 0.006 {
                d = (d + 0.35).min(1.0);
            }
            if web {
                let cu = if wc.0 { u } else { 1.0 - u };
                let cv = if wc.1 { v } else { 1.0 - v };
                let rr = (cu * cu + cv * cv).sqrt();
                if rr < 0.35 {
                    let a = cv.atan2(cu);
                    let radial = (1.0 - ((a * 9.0).sin().abs() * 40.0 * rr).min(1.0)) * 0.5;
                    let ring = (1.0 - (((rr * 60.0).sin().abs()) * 6.0).min(1.0)) * 0.35;
                    d = (d + (radial + ring) * (1.0 - rr / 0.35)).min(1.0);
                }
            }
            [d, d, d, 1.0]
        });
    })
}
