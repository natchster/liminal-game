//! Batch `t10`'s props in this crate, made real (`docs/PROPS.md` P3): the
//! carved cuckoo clock and the fireplace; their placers and what only they
//! use (`assets/src/blender/props/t10.py`, `t10.notes.md`).
use super::*;
use game::assets::props::prop_part_as_modelled;

/// The Blender part `name`, as modelled, in `grp` with `mat`
fn t10_put(grp: &Obj, name: &str, mat: &Mat) -> Option<Obj> {
    let m = mesh(prop_part_as_modelled(name)?, mat);
    grp.add(&m);
    Some(m)
}

/// Something of where a thing was put, to vary what its painters paint
/// (a painter's own stream starts the same on every floor)
fn t10_vary(x: f64, z: f64, k: f64) -> u32 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for v in [x, z, k] {
        h ^= v.to_bits();
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    (h ^ (h >> 29) ^ (h >> 47)) as u32
}

// ---- painting kit (per pixel, pure: safe for `par_rows`) ----
fn t10_hash(seed: u32, i: i32, j: i32) -> f64 {
    let mut h = seed ^ (i as u32).wrapping_mul(0x27d4_eb2d) ^ (j as u32).wrapping_mul(0x1656_67b1);
    h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
    h = (h ^ (h >> 13)).wrapping_mul(0xc2b2_ae35);
    h ^= h >> 16;
    f64::from(h) / 4_294_967_296.0
}

fn t10_vn(seed: u32, x: f64, y: f64) -> f64 {
    let (xf, yf) = (x.floor(), y.floor());
    let (i, j) = (xf as i32, yf as i32);
    let (u, v) = (x - xf, y - yf);
    let (u, v) = (u * u * (3.0 - 2.0 * u), v * v * (3.0 - 2.0 * v));
    let (a, b) = (t10_hash(seed, i, j), t10_hash(seed, i + 1, j));
    let (c, d) = (t10_hash(seed, i, j + 1), t10_hash(seed, i + 1, j + 1));
    a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v
}

fn t10_fbm(seed: u32, x: f64, y: f64, oct: u32) -> f64 {
    let (mut s, mut amp, mut f, mut n) = (0.0, 0.5, 1.0, 0.0);
    for k in 0..oct {
        s += amp * t10_vn(seed.wrapping_add(k * 7919), x * f, y * f);
        n += amp;
        amp *= 0.5;
        f *= 2.03;
    }
    s / n
}

fn t10_ss(e0: f64, e1: f64, x: f64) -> f64 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn t10_mix(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

/// Fills the canvas pixel by pixel on every core: `f(u, v)` (u right, v up,
/// both 0..1, as the texture is laid) gives straight sRGB and alpha, 0..1
fn t10_fill(ctx: &mut Canvas, w: f64, h: f64, f: impl Fn(f64, f64) -> [f64; 4] + Sync) {
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

fn t10_opaque(c: [f64; 3]) -> [f64; 4] {
    [c[0], c[1], c[2], 1.0]
}

// ================================================== the cuckoo clock

/// The clock (t10.py `CK_*`): where its dial's middle hangs (y), the
/// dial's face (z), the pendulum's hook under the case
const CK_DIAL_Y: f64 = 1.70;
const CK_DIAL_FACE: f64 = 0.008 + 0.110 + 0.004;
const CK_PEND: (f64, f64, f64) = (0.0, -0.020 - 0.100, 0.030);

// R741 ornate cuckoo clock: 3% chance per level. A carved Black Forest
// cuckoo clock of the traditional kind (Blender batch `t10`): oak leaves
// and acorns carved round its dial, a bird with its wings spread on its
// top, the cuckoo's door shut; a dark wooden dial with bone numerals and
// hands, stopped; a carved leaf on its pendulum, its two cast pine-cone
// weights on brass chains. Linden stained dark, the stain rubbed from the
// carving's high points, dust in its hollows.
pub(super) fn place_ornate_cuckoo_clocks(g: &mut Game, _dist: &[i32]) {
    g.pp.cuckoo_clock_positions_r741.clear(); // R741 reset per level
    if rnd() < 0.97 {
        return;
    }
    let wt = wall_tiles(g);
    if wt.is_empty() {
        return;
    }
    let (x, z, dx, dz) = wt[(rnd() * wt.len() as f64).floor() as usize];
    let cc = group();
    cc.rotation().y = face_wall(dx, dz);
    cc.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.015), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.015));
    let p = cc.pos();
    let vary = t10_vary(p.x, p.z, CK_DIAL_Y);
    // the parts are modelled with the wall at z = 0 and the dial's middle at
    // y = 0: on the wall, 15 mm behind the spot
    let body = group();
    body.position().set(0, CK_DIAL_Y, -0.015);
    cc.add(&body);
    t10_put(&body, "Real__cuckooCase", &lambert!({ map: &ck_wood_tex(vary, 0.0) }));
    t10_put(&body, "Real__cuckooCarving", &lambert!({ map: &ck_wood_tex(vary ^ 0x2b, 0.35) }));
    t10_put(&body, "Real__cuckooDial", &lambert!({ map: &ck_dial_tex(vary) }));
    // the hands, stopped where the old ones were (the old draws)
    let hour_ang = rnd() * PI * 2.0 - PI / 2.0;
    let min_ang = rnd() * PI * 2.0 - PI / 2.0;
    let bone = lambert!({ color: 0xc8b996 });
    for (name, ang, dz) in [("Real__cuckooHour", hour_ang, 0.0004), ("Real__cuckooMinute", min_ang, 0.0022)] {
        if let Some(h) = t10_put(&body, name, &bone) {
            h.position().set(0, 0, CK_DIAL_FACE + dz);
            h.rotation().z = ang - PI / 2.0;
        }
    }
    let brass = phong!({ color: 0x6e5a32, specular: 0x5a4a2a, shininess: 40 });
    t10_put(&body, "Real__cuckooChains", &brass);
    t10_put(&body, "Real__cuckooWeights", &phong!({ map: &ck_iron_tex(vary), specular: 0x1e1e1e, shininess: 20 }));
    // the pendulum hangs from its hook under the case
    let pendulum_group = group();
    pendulum_group.position().set(CK_PEND.0, CK_PEND.1, CK_PEND.2);
    t10_put(&pendulum_group, "Real__cuckooPendRod", &brass);
    t10_put(&pendulum_group, "Real__cuckooPendBob", &lambert!({ map: &ck_wood_tex(vary ^ 0x55, 0.35) }));
    body.add(&pendulum_group);
    // 50% chance: pendulum swing animator (R741: tracked for cuckoo-call audio).
    let is_swinging = rnd() < 0.5;
    if is_swinging {
        let swing_phase = rnd() * PI * 2.0;
        let pg2 = pendulum_group.clone();
        g.animate(move |_g, _dt, t| {
            pg2.rotation().z = (t * PI * 1.4 + swing_phase).sin() * 0.20;
        });
    }
    g.world.add(&cc);
    if is_swinging {
        let p = cc.pos();
        g.pp.cuckoo_clock_positions_r741.push(v3(p.x, 0, p.z)); // R741
    }
}

/// Linden stained dark walnut, as the clock's front is mapped (u across, v
/// up): the stain's streaks, darker in the hollows and the dust lodged
/// there, rubbed lighter where it stands out (`rub`: the carving's more)
fn ck_wood_tex(vary: u32, rub: f64) -> Tex {
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        let seed = vary ^ 0xc0c;
        t10_fill(ctx, w, h, |u, v| {
            let g = t10_fbm(seed, u * 4.0, v * 30.0, 3);
            let fine = t10_vn(seed ^ 3, u * 90.0, v * 400.0);
            let mut o = [0.17, 0.095, 0.05].map(|c| c * (0.8 + 0.35 * g + 0.1 * fine));
            // the knife's facets and the stain rubbed off them
            let facet = t10_fbm(seed ^ 5, u * 40.0, v * 40.0, 2);
            o = t10_mix(o, [0.42, 0.29, 0.17], rub * t10_ss(0.55, 0.75, facet));
            // dust in the hollows, a grey bloom over all of it
            let dust = 0.04 + 0.12 * t10_ss(0.4, 0.75, t10_fbm(seed ^ 7, u * 14.0, v * 14.0, 3));
            t10_opaque(t10_mix(o, [0.40, 0.37, 0.33], dust))
        });
    })
}

/// The dial (100 mm, as its face is mapped): dark wood, a turned ring,
/// Roman numerals of bone set round it, the minute dots between
fn ck_dial_tex(vary: u32) -> Tex {
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        let seed = vary ^ 0xd1a1;
        t10_fill(ctx, w, h, |u, v| {
            let g = t10_fbm(seed, u * 3.0, v * 24.0, 3);
            let (x, y) = (u - 0.5, v - 0.5);
            let rr = (x * x + y * y).sqrt();
            let mut o = [0.17, 0.10, 0.06].map(|c| c * (0.8 + 0.4 * g));
            // the turned rings
            if (rr - 0.43).abs() < 0.006 || (rr - 0.25).abs() < 0.004 {
                o = o.map(|c| c * 0.6);
            }
            t10_opaque(t10_mix(o, [0.42, 0.40, 0.35], 0.12))
        });
        let (cx, cy) = (w / 2.0, h / 2.0);
        const NUM: [&str; 12] = ["XII", "I", "II", "III", "IIII", "V", "VI", "VII", "VIII", "IX", "X", "XI"];
        ctx.set_fill_style("rgba(206,192,156,0.95)");
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        ctx.set_font("bold 24px Times New Roman");
        for (k, n) in NUM.iter().enumerate() {
            let a = k as f64 * PI / 6.0;
            ctx.save();
            ctx.translate(cx + a.sin() * 86.0, cy - a.cos() * 86.0);
            ctx.rotate(a);
            ctx.scale(0.8, 1.0);
            ctx.fill_text(n, 0, 0);
            ctx.restore();
        }
        for k in 0..60 {
            if k % 5 == 0 {
                continue;
            }
            let a = k as f64 * PI / 30.0;
            ctx.begin_path();
            ctx.arc(cx + a.sin() * 112.0, cy - a.cos() * 112.0, 1.4, 0, 7);
            ctx.fill();
        }
        // the bone yellowed and grimed, unevenly
        let mut r = SplitMix64::new(vary, 0xd1a2);
        for _ in 0..6 {
            let (x, y, rr) = (r.next() * w, r.next() * h, 20.0 + r.next() * 50.0);
            let gr = ctx.create_radial_gradient(x, y, 0, x, y, rr);
            gr.add_color_stop(0, "rgba(60,44,24,0.25)");
            gr.add_color_stop(1, "rgba(60,44,24,0)");
            ctx.set_fill_style(gr);
            ctx.fill_rect(x - rr, y - rr, rr * 2.0, rr * 2.0);
        }
    })
}

/// Cast iron gone dull, rust coming through (the weights)
fn ck_iron_tex(vary: u32) -> Tex {
    canvas_tex_opt(64, 128, false, move |ctx, w, h| {
        let seed = vary ^ 0x1e0;
        t10_fill(ctx, w, h, |u, v| {
            let n = t10_fbm(seed, u * 8.0, v * 16.0, 3);
            let rust = t10_ss(0.55, 0.75, t10_fbm(seed ^ 3, u * 12.0, v * 24.0, 3));
            t10_opaque(t10_mix([0.08 + 0.04 * n, 0.075 + 0.035 * n, 0.07 + 0.03 * n], [0.30, 0.17, 0.09], rust * 0.8))
        });
    })
}

// ===================================================== the fireplace

/// The fireplace (t10.py `FP_*`): the basket's floor (y) and the depth (z)
/// the logs lie at, its shelf's top (y)
const FP_BASKET_Y: f64 = 0.130;
const FP_LOG_Z: f64 = 0.080;
const FP_SHELF_TOP: f64 = 1.100;

/// The charred ends left in the basket: (part, radius)
const FP_LOGS: [(&str, f64); 4] =
    [("Real__fireplaceLogA", 0.040), ("Real__fireplaceLogB", 0.034), ("Real__fireplaceLogC", 0.030), ("Real__fireplaceLogD", 0.026)];

// R744 fireplace: 2.5% chance per level. A Victorian bedroom fireplace
// against the wall (Blender batch `t10`): a painted pine mantelpiece on its
// plinths, its corbels, frieze and shelf, the paint gone yellow and chipped
// through its coats to the wood; a cast-iron arched register insert, its
// black-lead gone dull and rusting, its canopy, basket and fret; the
// firebrick behind sooted black; a slate hearth. Cold a long time: ash in
// the basket and fallen through it, the charred ends of the last fire, the
// soot of years climbed the wall above the shelf.
pub(super) fn place_fireplaces(g: &mut Game, _dist: &[i32]) {
    g.pp.lit_fireplace_positions.clear(); // R744 reset per level
    if rnd() < 0.975 {
        return;
    }
    let wt = wall_tiles(g);
    if wt.is_empty() {
        return;
    }
    let (x, z, dx, dz) = wt[(rnd() * wt.len() as f64).floor() as usize];
    // the old draws, call for call: how many logs and where each lay (now
    // the charred ends of the last fire), whether it was lit and its flames
    // (it is cold: they are drawn and let go)
    let log_count = 3 + (rnd() * 2.0).floor() as i32;
    let log_x: Vec<f64> = (0..log_count).map(|_| (rnd() - 0.5) * 0.02).collect();
    let is_lit = rnd() < 0.55;
    if is_lit {
        let flame_colors = [0xffe040, 0xffa020, 0xff6010];
        for _ in 0..5 {
            let _ = choice(&flame_colors); // (a flame's colour)
            let _ = rnd(); // (its place across)
            let _ = rnd(); // (and in depth)
        }
        let _ = rnd(); // (the flicker's phases)
        let _ = rnd();
    }
    let fp = group();
    // Back (local z 0) against the wall, front towards the room.
    fp.rotation().y = face_wall(dx, dz);
    fp.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.005), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.005));
    let p = fp.pos();
    let vary = t10_vary(p.x, p.z, log_count as f64);
    // the parts are modelled with the wall at z = 0: 5 mm behind the spot
    let body = group();
    body.position().z = -0.005;
    fp.add(&body);
    t10_put(&body, "Real__fireplaceMantel", &lambert!({ map: &fp_paint_tex(vary) }));
    let iron = phong!({ map: &fp_iron_tex(vary, 0.25), specular: 0x262626, shininess: 22 });
    t10_put(&body, "Real__fireplaceInsert", &iron);
    t10_put(&body, "Real__fireplaceGrate", &phong!({ map: &fp_iron_tex(vary ^ 0x9, 0.55), specular: 0x1a1a1a, shininess: 16 }));
    t10_put(&body, "Real__fireplaceFirebox", &lambert!({ map: &fp_soot_tex(vary) }));
    t10_put(&body, "Real__fireplaceHearth", &lambert!({ map: &fp_slate_tex(vary) }));
    t10_put(&body, "Real__fireplaceAsh", &lambert!({ map: &fp_ash_tex(), color: 0x9c988f }));
    // the charred ends, lying across the basket on the ash
    let char_mat = lambert!({ map: &fp_char_tex() });
    const SLOTS: [(f64, f64); 4] = [(-0.024, 0.0), (0.030, 0.0), (0.004, 1.0), (-0.060, 0.0)];
    for (i, &jx) in log_x.iter().enumerate() {
        let (name, r) = FP_LOGS[i];
        let (sz, layer) = SLOTS[i];
        if let Some(m) = t10_put(&body, name, &char_mat) {
            m.position().set(jx * 3.0, FP_BASKET_Y + 0.018 + r + layer * 0.050, FP_LOG_Z + sz);
            m.rotation().y = (t10_hash(vary, i as i32, 1) - 0.5) * 0.35;
            m.rotation().x = t10_hash(vary, i as i32, 2) * PI * 2.0;
        }
    }
    // the soot of years on the wall above the shelf
    let soot = mesh(
        plane_geo!(0.90, 0.80),
        lambert!({ map: &fp_ghost_tex(vary), transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 }),
    );
    soot.position().set(0, FP_SHELF_TOP + 0.40 - 0.02, 0.002);
    body.add(&soot);
    g.world.add(&fp);
}

/// The mantelpiece's paint as its front is mapped (u across its 960 mm, v
/// up its 1100): cream gloss gone yellow and crazed, chipped through to an
/// older green coat and to the pine, grimed low down, soot over the
/// opening's head
fn fp_paint_tex(vary: u32) -> Tex {
    canvas_tex_opt(256, 296, false, move |ctx, w, h| {
        let seed = vary ^ 0xfa1;
        t10_fill(ctx, w, h, |u, v| {
            let (x, y) = (u * 0.96, v * 1.10);
            let m = t10_fbm(seed, x * 6.0, y * 6.0, 4);
            let mut o = [0.66, 0.61, 0.50].map(|c| c * (0.9 + 0.14 * m));
            // crazing: a fine net of dark lines
            let cr = t10_vn(seed ^ 3, x * 180.0, y * 180.0);
            if (cr - 0.5).abs() < 0.03 {
                o = o.map(|c| c * 0.82);
            }
            // chips: through the cream to the green, deeper to the pine
            let ch = t10_fbm(seed ^ 5, x * 70.0, y * 70.0, 3)
                + 0.10 * (1.0 - t10_ss(0.0, 0.25, y))
                + 0.06 * (1.0 - t10_ss(0.0, 0.05, (u - 0.5).abs().min(0.5 - (u - 0.5).abs())));
            if ch > 0.71 {
                o = [0.36, 0.38, 0.32];
            }
            if ch > 0.76 {
                o = [0.48, 0.34, 0.21].map(|c| c * (0.85 + 0.2 * m));
            }
            // grime: kicked at its foot, handled at the shelf's edge
            o = o.map(|c| c * (1.0 - 0.35 * (1.0 - t10_ss(0.0, 0.2, y))));
            // the soot that crept out over the opening's head onto the frieze
            let sx = (u - 0.5).abs() * 0.96;
            let soot = (1.0 - t10_ss(0.18, 0.36, sx)) * t10_ss(0.80, 0.90, y) * (1.0 - t10_ss(1.02, 1.08, y));
            o = t10_mix(o, [0.10, 0.09, 0.08], soot * (0.55 + 0.3 * m));
            t10_opaque(t10_mix(o, [0.47, 0.45, 0.40], 0.10))
        });
    })
}

/// Cast iron (as the insert's front is mapped): black-lead gone dull and
/// grey, rust (`rust`: how much) blooming through, most at its foot
fn fp_iron_tex(vary: u32, rust: f64) -> Tex {
    canvas_tex_opt(256, 320, false, move |ctx, w, h| {
        let seed = vary ^ 0x1a0;
        t10_fill(ctx, w, h, |u, v| {
            let n = t10_fbm(seed, u * 10.0, v * 12.0, 4);
            let mut o = [0.075 + 0.05 * n, 0.072 + 0.046 * n, 0.068 + 0.042 * n];
            let r = t10_fbm(seed ^ 3, u * 16.0, v * 20.0, 4) + rust * 0.35 * (1.0 - t10_ss(0.0, 0.35, v)) + rust * 0.2;
            o = t10_mix(o, [0.27, 0.15, 0.08], 0.75 * t10_ss(0.70, 0.86, r));
            let dust = 0.10 + 0.12 * t10_fbm(seed ^ 7, u * 5.0, v * 5.0, 2);
            t10_opaque(t10_mix(o, [0.40, 0.38, 0.34], dust))
        });
    })
}

/// The firebrick behind, sooted: its courses just showing through the black
fn fp_soot_tex(vary: u32) -> Tex {
    canvas_tex_opt(128, 256, false, move |ctx, w, h| {
        let seed = vary ^ 0x5007;
        t10_fill(ctx, w, h, |u, v| {
            let (x, y) = (u * 0.44, v * 0.76);
            let row = (y / 0.065).floor();
            let bx = (x + if row as i32 % 2 == 0 { 0.0 } else { 0.11 }) / 0.22;
            let joint = (y / 0.065).fract() < 0.08 || bx.fract() < 0.04;
            let brick = [0.36, 0.24, 0.17].map(|c| c * (0.8 + 0.3 * t10_hash(seed, bx.floor() as i32, row as i32)));
            let mut o = if joint { [0.20, 0.18, 0.16] } else { brick };
            // the soot: thick everywhere, thickest up the back where the smoke went
            let s = 0.82 + 0.15 * t10_ss(0.1, 0.8, v) * (1.0 - (u - 0.5).abs()) + 0.08 * t10_fbm(seed ^ 3, u * 9.0, v * 9.0, 3);
            o = t10_mix(o, [0.025, 0.022, 0.02], s.min(0.97));
            t10_opaque(o)
        });
    })
}

/// The slate hearth (top as mapped): its cleft, its grey, ash ground in,
/// a crack across it
fn fp_slate_tex(vary: u32) -> Tex {
    canvas_tex_opt(256, 128, false, move |ctx, w, h| {
        let seed = vary ^ 0x51a7;
        let crack_y = 0.3 + 0.4 * t10_hash(seed, 1, 1);
        t10_fill(ctx, w, h, |u, v| {
            let n = t10_fbm(seed, u * 12.0, v * 4.0, 4);
            let mut o = [0.16, 0.17, 0.18].map(|c| c * (0.85 + 0.3 * n));
            // ash trodden in, heaviest by the fire
            let ash = t10_ss(0.45, 0.75, t10_fbm(seed ^ 3, u * 7.0, v * 7.0, 3)) * (0.4 + 0.6 * v);
            o = t10_mix(o, [0.48, 0.46, 0.43], 0.5 * ash);
            let cy = crack_y + 0.04 * (t10_fbm(seed ^ 5, u * 6.0, 0.0, 3) - 0.5);
            if (v - cy).abs() < 0.006 && u > 0.2 && u < 0.85 {
                o = o.map(|c| c * 0.35);
            }
            t10_opaque(o)
        });
    })
}

/// Wood ash: pale grey, flecked with char
fn fp_ash_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            canvas_tex_opt(128, 128, false, |ctx, w, h| {
                t10_fill(ctx, w, h, |u, v| {
                    let n = t10_fbm(0xa5, u * 12.0, v * 12.0, 4);
                    let fleck = if t10_hash(0xa6, (u * 128.0) as i32, (v * 128.0) as i32) > 0.93 { 0.3 } else { 1.0 };
                    let k = (0.38 + 0.16 * n) * fleck;
                    [k, k * 0.98, k * 0.94, 1.0]
                });
            })
        })
        .clone()
    })
}

/// Charcoal: black, crazed into blocks, their edges burnt to white ash
fn fp_char_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            canvas_tex_opt(128, 128, false, |ctx, w, h| {
                t10_fill(ctx, w, h, |u, v| {
                    let (gx, gy) = (u * 10.0 + 0.3 * t10_vn(0xc1, u * 8.0, v * 8.0), v * 6.0);
                    let edge = (gx.fract() - 0.5).abs().max((gy.fract() - 0.5).abs());
                    let mut k = 0.035 + 0.03 * t10_vn(0xc2, u * 40.0, v * 40.0);
                    if edge > 0.44 {
                        k = 0.01;
                    } else if edge > 0.38 {
                        k = 0.30;
                    }
                    let ash = t10_ss(0.6, 0.8, t10_fbm(0xc3, u * 6.0, v * 6.0, 3));
                    let k = k + (0.42 - k) * ash * 0.7;
                    [k, k * 0.96, k * 0.92, 1.0]
                });
            })
        })
        .clone()
    })
}

/// The soot of years on the wall above the shelf: a dark bloom rising from
/// the shelf's edge, fading up the wall, darkest over the middle
fn fp_ghost_tex(vary: u32) -> Tex {
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        let seed = vary ^ 0x6057;
        t10_fill(ctx, w, h, |u, v| {
            let x = (u - 0.5) * 2.0;
            let n = t10_fbm(seed, u * 5.0, v * 3.0, 4);
            let a = (1.0 - t10_ss(0.0, 0.95, v + 0.25 * (n - 0.5))) * (1.0 - t10_ss(0.35, 1.0, x.abs() + 0.2 * v)) * (0.55 + 0.45 * n);
            [0.06, 0.055, 0.05, (a * 0.55).min(0.6)]
        });
    })
}
