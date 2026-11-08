//! Batch `r11a`'s props in this crate, made real (`docs/PROPS.md` P2): its
//! placers and what only they use.
use super::*;

// Water puddle: 8% chance per level. Where a drip from the ceiling
// lands, and has gone on landing (`assets/src/blender/props/r11a.notes.md`):
// on carpet the water has soaked in, a dark patch with a soft ragged edge and
// a darker rim where the dirt wicked out to it, and where the drops land the
// pile is so wet that water stands on it, with a sheen; on a hard floor it
// stands, a thin film darker than the floor round it, its edge sharp and
// lobed, the dried tide lines of where it reached before outside it, and a
// sheen. Each drop sends a ring out across the standing water, gone within a
// second. Painted on transparent planes flat on the floor, as wide as the old
// disc.
pub(crate) fn place_water_puddles(g: &mut Game, _dist: &[i32]) {
    g.pp.puddle_positions.clear(); // reset per level
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let puddle = group();
    // the old disc's radius: the plane is as wide as the disc was, the water
    // inside it
    let p_radius = 0.10 + rnd() * 0.10;
    let floor = puddle_floor_kind(g, x, z);
    let hard = floor != 0;
    let vary = vary_by(x, z, (p_radius * 1.0e4) as i32);
    let mut layers = vec![water_puddle_mat(p_radius, hard, false, vary)];
    if !hard {
        // the water standing on the soaked pile where the drops land
        layers.push(water_puddle_mat(p_radius, false, true, vary));
    }
    for (k, mat) in layers.iter().enumerate() {
        let geo = plane_geo!(p_radius * 2.0, p_radius * 2.0);
        geo.rotate_x(-PI / 2.0);
        let water = mesh(geo, mat);
        water.position().y = 0.003 + k as f64 * 0.0005;
        puddle.add(&water);
    }
    // the ring a drop sends out (the old torus): a soft bright crest and its
    // trough, on a plane over the water
    let ring_mat = drip_ring_mat();
    let rgeo = plane_geo!(p_radius * 0.6, p_radius * 0.6);
    rgeo.rotate_x(-PI / 2.0);
    let ripple = mesh(rgeo, &ring_mat);
    ripple.position().y = 0.0042;
    ripple.set_visible(false);
    puddle.add(&ripple);
    // the old ripple's phase and speed: when each drop lands (every 5-8 s)
    let ripple_phase = rnd() * PI * 2.0;
    let ripple_speed = 0.5 + rnd() * 0.3;
    {
        let ripple = ripple.clone();
        // the ring's radius on the plane at a scale of one (its crest at 0.4
        // of the plane's width), and how far the standing water reaches
        let r1 = p_radius * 0.6 * DRIP_RING_AT;
        let reach = if hard { p_radius * 0.62 } else { p_radius * 0.26 };
        g.animate(move |_g, _dt, t| {
            // seconds since the last drop landed; the ring spreads at about
            // the speed of the slowest ripples on water (0.23 m/s) and dies
            // away as it goes
            let since = ((t * ripple_speed + ripple_phase) % 4.0) / ripple_speed;
            let r = 0.008 + since * 0.23;
            let life = (1.0 - r / reach).min(1.0 - since / 1.1);
            if life <= 0.0 {
                ripple.set_visible(false);
                return;
            }
            ripple.set_visible(true);
            ripple.scale().set_scalar(r / r1);
            ripple.mat().set_opacity(life.min(1.0).powf(1.5));
        });
    }
    scatter_on_floor(g, &puddle, x, z, 2.4);
    g.world.add(&puddle);
    // Track for drip audio.
    let p = puddle.pos();
    g.pp.puddle_positions.push(v3(p.x, 0.003, p.z));
}

/// Where the drip ring's crest lies on its plane (a fraction of its width)
const DRIP_RING_AT: f64 = 0.4;

/// What the floor of tile (x, z) is: 0 carpet (or carpet tiles), 1 concrete,
/// 2 glazed tile
fn puddle_floor_kind(g: &Game, x: i32, z: i32) -> u8 {
    if g.region_flags_at(x, z) & game::visuals::lighting::RF_MOSS != 0 {
        return 0;
    }
    match g.zone_ref(x, z).map(|zn| zn.floor_mat) {
        Some("concrete") => 1,
        Some("tile") => 2,
        _ => 0,
    }
}

/// Water a drip has left on the floor, on a plane `2r` square: on a hard
/// floor a film with a sharp lobed edge, a damp margin, the dried rims of
/// where it reached before and a pool or two beside it; on carpet the patch
/// it soaked (or, `standing`, the water standing on the pile in the middle
/// of it). Its material, with the sheen of wet things.
fn water_puddle_mat(r: f64, hard: bool, standing: bool, vary: u32) -> Mat {
    let pw = r * 2.0;
    let wet = if hard {
        [0.1, 0.104, 0.11]
    } else if standing {
        [0.07, 0.062, 0.055]
    } else {
        [0.12, 0.1, 0.08]
    };
    let tex = mark_tex(192.0, 192.0, wet, vary, move |ctx, cw, ch| {
        let seed = (rnd() * 4.0e9) as u32;
        let s = pw / cw;
        let aa = s * 1.2;
        // how far the water reaches now, and before (the dried rims)
        let reach = if hard {
            r * (0.6 + 0.08 * rnd())
        } else if standing {
            r * (0.3 + 0.08 * rnd())
        } else {
            r * (0.74 + 0.08 * rnd())
        };
        let rims: Vec<(f64, u32, f64)> = if hard {
            (0..1 + (rnd() * 2.2) as usize)
                .map(|k| (reach * (1.12 + 0.12 * k as f64 + 0.06 * rnd()), seed ^ (0x51 + k as u32), 0.1 + 0.12 * rnd()))
                .collect()
        } else {
            vec![]
        };
        let sats: Vec<(f64, f64, f64)> = if hard {
            (0..(rnd() * 3.0) as usize)
                .map(|_| {
                    let (a, rr) = (rnd() * PI * 2.0, reach * (0.95 + 0.25 * rnd()));
                    (a.cos() * rr, a.sin() * rr, reach * (0.08 + 0.12 * rnd()))
                })
                .collect()
        } else {
            vec![]
        };
        // the outline of a wetting that reached `rr` (signed distance, m):
        // lobed where the floor dips, finer on a hard floor, where the edge
        // stops at every grain
        let fine = if hard { 0.06 } else { 0.02 };
        let outline = move |x: f64, y: f64, rr: f64, sd: u32| -> f64 {
            let rho = x.hypot(y) / rr;
            let (cu, cv) = if rho > 1e-6 { (x / rr / rho, y / rr / rho) } else { (1.0, 0.0) };
            let lobe = 1.0
                + 0.42 * (fbm(sd, cu * 1.2 + 4.0, cv * 1.2 + 4.0, 3) - 0.5)
                + 0.14 * (vnoise(sd ^ 0x9, cu * 4.0 + 2.0, cv * 4.0 + 2.0) - 0.5)
                + fine * (vnoise(sd ^ 0xa, cu * 10.0 + 1.0, cv * 10.0 + 1.0) - 0.5);
            (rho - lobe) * rr
        };
        let mut img = ctx.create_image_data(cw, ch);
        let n = cw as usize;
        par_rows(&mut img.data, n * 4, |j, out| {
            let y = (j as f64 + 0.5) * s - r;
            for i in 0..n {
                let x = (i as f64 + 0.5) * s - r;
                let mut px = Px::CLEAR;
                let mut d = outline(x, y, reach, seed);
                for &(qx, qz, qr) in &sats {
                    d = smin(d, (x - qx).hypot(y - qz) - qr, 0.01);
                }
                if hard {
                    d += (vnoise(seed ^ 0x4d, x * 90.0, y * 90.0) - 0.5) * 0.003;
                    // where it reached before: a dried rim of what it carried
                    for &(rr, sd, k) in &rims {
                        let dr = outline(x, y, rr, sd) + (vnoise(sd, x * 70.0, y * 70.0) - 0.5) * 0.004;
                        px.lay([0.2, 0.17, 0.13], tide(dr, aa, 0.004) * k * (0.5 + 0.5 * fbm(sd ^ 0x3, x * 20.0, y * 20.0, 2)));
                    }
                    // the damp margin, the film, the dark line of its edge
                    if d > 0.0 {
                        px.lay([0.11, 0.105, 0.1], 0.22 * (-d / 0.012).exp() * (0.6 + 0.4 * vnoise(seed ^ 0x5e, x * 50.0, y * 50.0)));
                    }
                    px.lay(wet, (0.5 - d / aa).clamp(0.0, 1.0) * (0.36 + 0.08 * vnoise(seed ^ 0x3c, x * 25.0, y * 25.0)));
                    px.lay([0.035, 0.035, 0.04], tide(d, aa, 0.0015) * 0.35);
                } else if standing {
                    // water on the pile: its edge soft over a few fibres
                    let dd = d + (vnoise(seed ^ 0x71, x * 160.0, y * 160.0) - 0.5) * 0.004;
                    px.lay(wet, sstep(0.004, -0.008, dd) * (0.26 + 0.08 * vnoise(seed ^ 0x3c, x * 40.0, y * 40.0)));
                } else {
                    // soaked in: the edge ragged and soft, a darker rim at it
                    let dd = d
                        + (fbm(seed ^ 0x77, x * 12.0, y * 12.0, 2) - 0.5) * 0.016
                        + (fbm(seed ^ 0x78, x * 45.0, y * 45.0, 2) - 0.5) * 0.01;
                    px.lay(wet, sstep(0.012, -0.024, dd) * (0.55 + 0.15 * vnoise(seed ^ 0x3c, x * 30.0, y * 30.0)));
                    px.lay([0.09, 0.07, 0.05], 0.24 * (-(dd + 0.004).abs() / 0.007).exp());
                }
                // never a square's edge
                px.a *= sstep(0.0, 0.025, r - x.abs().max(y.abs()));
                let c = px.rgba();
                for (k, v) in c.iter().enumerate() {
                    out[i * 4 + k] = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
                }
            }
        });
        ctx.put_image_data(&img, 0, 0);
    });
    let (sheen, shine) = if hard {
        (0.3, 40.0)
    } else if standing {
        (0.3, 34.0)
    } else {
        (0.03, 18.0)
    };
    let m = phong!({ map: &tex, transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2, shininess: shine });
    m.m().specular.set_rgb(sheen, sheen, sheen);
    m
}

/// The ring a drop sends across standing water, on a plane: its crest (the
/// lights' sheen caught on its slope) at `DRIP_RING_AT` of the width, the
/// trough inside it, and a fainter second ring behind.
fn drip_ring_mat() -> Mat {
    let tex = mark_tex(128.0, 128.0, [0.6, 0.62, 0.64], 0, |ctx, cw, ch| {
        let mut img = ctx.create_image_data(cw, ch);
        let n = cw as usize;
        par_rows(&mut img.data, n * 4, |j, out| {
            let v = (j as f64 + 0.5) / ch - 0.5;
            for i in 0..n {
                let u = (i as f64 + 0.5) / cw - 0.5;
                let q = u.hypot(v);
                let band = |at: f64, w: f64| (-((q - at) / w).powi(2)).exp();
                let mut px = Px::CLEAR;
                px.lay([0.04, 0.045, 0.05], 0.22 * band(DRIP_RING_AT - 0.035, 0.018) + 0.1 * band(DRIP_RING_AT - 0.14, 0.014));
                px.lay([0.62, 0.64, 0.66], 0.42 * band(DRIP_RING_AT, 0.014) + 0.16 * band(DRIP_RING_AT - 0.1, 0.012));
                let c = px.rgba();
                for (k, w) in c.iter().enumerate() {
                    out[i * 4 + k] = (w.clamp(0.0, 1.0) * 255.0).round() as u8;
                }
            }
        });
        ctx.put_image_data(&img, 0, 0);
    });
    lambert!({ map: &tex, transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -3, polygonOffsetUnits: -3 })
}

/// The smooth minimum of two distances, blended over `k`
fn smin(a: f64, b: f64, k: f64) -> f64 {
    let h = (0.5 + 0.5 * (b - a) / k).clamp(0.0, 1.0);
    b + (a - b) * h - k * h * (1.0 - h)
}

/// A pixel's straight sRGB colour and alpha, built up by laying marks over it
#[derive(Clone, Copy)]
struct Px {
    c: [f64; 3],
    a: f64,
}

impl Px {
    const CLEAR: Px = Px { c: [0.0; 3], a: 0.0 };
    /// `c` at alpha `a` laid over what is there
    fn lay(&mut self, c: [f64; 3], a: f64) {
        let a = a.clamp(0.0, 1.0);
        let out = a + self.a * (1.0 - a);
        if out > 1e-6 {
            for (k, v) in self.c.iter_mut().enumerate() {
                *v = (c[k] * a + *v * self.a * (1.0 - a)) / out;
            }
        }
        self.a = out;
    }
    fn rgba(self) -> [f64; 4] {
        [self.c[0], self.c[1], self.c[2], self.a]
    }
}

/// A transparent mark `w × h` px painted by `paint` on a stream of its own,
/// varied by `vary` (a painter's own stream starts the same on every floor:
/// `vary` comes from where the mark was put); where it is clear its colour is
/// `bleed` (sRGB), so its smaller mips keep the mark's colour at its soft
/// edges instead of darkening them
pub(super) fn mark_tex<F: FnOnce(&mut Canvas, f64, f64)>(w: f64, h: f64, bleed: [f64; 3], vary: u32, paint: F) -> Tex {
    let tex = canvas_tex_opt(w, h, false, move |ctx: &mut Canvas, cw: f64, ch: f64| {
        let seed = (rnd() * 4.0e9) as u32 ^ vary;
        with_own_stream(seed, || paint(ctx, cw, ch))
    });
    {
        let img = tex.image();
        let mut im = img.borrow_mut();
        for px in im.data.chunks_exact_mut(4) {
            let k = sstep(0.0, 0.3, f64::from(px[3]) / 255.0);
            for (c, b) in px.iter_mut().zip(bleed) {
                *c = (b * 255.0 + (f64::from(*c) - b * 255.0) * k).round().clamp(0.0, 255.0) as u8;
            }
        }
        im.version += 1;
    }
    tex
}

/// The decal material of a mark painted in `tex`, lit as the surface under it
fn mark_mat(tex: &Tex) -> Mat {
    lambert!({ map: tex, transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 })
}

/// What the wall facing tile (x, z) is: 0 paper or paint on plasterboard, 1
/// concrete or brick, 2 glazed tile
fn mark_wall_kind(g: &Game, x: i32, z: i32) -> u8 {
    if g.region_flags_at(x, z) & game::visuals::lighting::RF_MOSS != 0 {
        return 0;
    }
    match g.zone_ref(x, z).map(|zn| zn.wall_mat) {
        Some("concrete" | "brick") => 1,
        Some("tile") => 2,
        _ => 0,
    }
}

// Wall claw marks: 4% chance per level. Something dragged its nails down
// the wall (`assets/src/blender/props/r11a.notes.md`): three or four gouges
// side by side a finger apart, dug in hard where they start and lifting
// toward their ends, the nails skipping now and then. On paper or paint over
// plasterboard each tore the surface: the paint and the face paper ripped
// back in a ragged lip (its brown inner plies at the edge), the white gypsum
// beneath gouged out and greyed with the years, a strip of paper rolled up
// where a nail left off, gypsum dust fallen down the wall under them. On
// concrete or brick, pale scored furrows with chipped edges and their grit;
// on glazed tile, fine scratches through the glaze, flakes chipped out along
// them. Painted on a transparent plane the size of the old one.
pub(crate) fn place_claw_marks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    let surface = mark_wall_kind(g, c.x, c.z);
    let mat = claw_marks_mat(0.50, 0.70, surface, vary_by(c.x * 4 + c.dx, c.z * 4 + c.dz, 0x516));
    let decal = mesh(plane_geo!(0.50, 0.70), &mat);
    mount_on_wall(g, &decal, &c, 0.008, 1.4, 0.5);
    g.world.add(&decal);
}

/// One gouge of a claw mark: its centreline (m, x across the plane, y down
/// from its top), how wide it is along it (half width, m) and how deep (0..1)
struct Gouge {
    pts: Vec<(f64, f64)>,
    half: Vec<f64>,
    depth: Vec<f64>,
}

/// Claw marks gouged into a wall of `surface` (`mark_wall_kind`), painted on
/// a plane `pw × ph` m
fn claw_marks_mat(pw: f64, ph: f64, surface: u8, vary: u32) -> Mat {
    let (tw, th) = ((pw / 0.001).round(), (ph / 0.001).round());
    // the colours: what lies under the surface, its torn edge, the dust
    let (core, edge, dust) = match surface {
        1 => ([0.8, 0.78, 0.74], [0.84, 0.82, 0.78], [0.82, 0.81, 0.77]),
        2 => ([0.82, 0.8, 0.76], [0.9, 0.89, 0.86], [0.82, 0.81, 0.78]),
        _ => ([0.86, 0.84, 0.79], [0.82, 0.78, 0.68], [0.86, 0.85, 0.81]),
    };
    // the face paper's brown inner plies, where the tear goes through them
    let plies = [0.6, 0.5, 0.37];
    let tex = mark_tex(tw, th, core, vary, move |ctx, cw, ch| {
        let seed = (rnd() * 4.0e9) as u32;
        {
            let s = pw / cw;
            let aa = s * 1.1;
            // the hand: three or four nails a finger apart, dragged down at a lean
            let n = if rnd() < 0.7 { 4 } else { 3 };
            let spacing = 0.02 + 0.008 * rnd();
            let lean = (0.1 + 0.35 * rnd()) * if rnd() < 0.5 { -1.0 } else { 1.0 };
            let len = ph * (0.52 + 0.18 * rnd());
            let bend = (rnd() - 0.5) * 0.25;
            let fan = 0.1 + 0.25 * rnd();
            let (dx, dy) = (lean.sin(), lean.cos());
            let (nx, ny) = (dy, -dx);
            let top = (pw / 2.0 - dx * len / 2.0, (ph - dy * len) / 2.0 - 0.02);
            // each finger's reach, where it starts and how hard it pressed
            let reach = [0.82, 1.0, 0.96, 0.72];
            let stagger = [0.016, 0.0, 0.006, 0.03];
            let press = [0.75, 1.0, 0.92, 0.6];
            let (wide, deep) = match surface {
                1 => (0.0048, 0.6),
                2 => (0.002, 0.5),
                _ => (0.0085, 1.0),
            };
            let gouges: Vec<Gouge> = (0..n)
                .map(|k| {
                    let f = if n == 3 { k + 1 } else { k };
                    let off = (k as f64 - (n as f64 - 1.0) / 2.0) * spacing;
                    let gs = seed.wrapping_add(31 * k as u32);
                    let (l0, l1) = (stagger[f] + 0.01 * rnd(), len * reach[f] * (0.9 + 0.15 * rnd()));
                    let hw = wide * (0.8 + 0.4 * rnd()) * (0.6 + 0.4 * press[f]);
                    let m = 72;
                    let mut g = Gouge { pts: vec![], half: vec![], depth: vec![] };
                    for i in 0..=m {
                        let t = i as f64 / m as f64;
                        let a = l0 + (l1 - l0) * t;
                        let o = off * (1.0 + fan * t) + bend * len * (t * t - t) + (fbm(gs, t * 6.0, 0.5, 2) - 0.5) * 0.004;
                        g.pts.push((top.0 + dx * a + nx * o, top.1 + dy * a + ny * o));
                        // dug in at once, lifting slowly; chattering, skipping
                        let mut e = sstep(0.0, 0.04, t) * (1.0 - t).powf(0.6) * (0.8 + 0.2 * vnoise(gs ^ 0x5, t * 40.0, 0.5));
                        if vnoise(gs ^ 0x6, t * 9.0, 0.5) > 0.82 {
                            e *= 0.3;
                        }
                        let e = e * press[f];
                        g.half.push(hw * e.sqrt().max(0.0));
                        g.depth.push(e * deep);
                    }
                    g
                })
                .collect();
            // strips of face paper rolled up where some nails left off (paper only)
            let curls: Vec<(f64, f64, f64, f64)> = if surface == 0 {
                gouges
                    .iter()
                    .filter(|_| rnd() < 0.6)
                    .map(|g| {
                        let k = g.pts.len() * (5 + (rnd() * 4.0) as usize) / 10;
                        let (p, q) = (g.pts[k], g.pts[(k + 4).min(g.pts.len() - 1)]);
                        (p.0, p.1, (q.1 - p.1).atan2(q.0 - p.0), 0.007 + 0.01 * rnd())
                    })
                    .collect()
            } else {
                vec![]
            };
            // each gouge's bounds, with the reach of its dust below it
            let bounds: Vec<[f64; 4]> = gouges
                .iter()
                .map(|g| {
                    let (mut b0, mut b1, mut b2, mut b3) = (f64::INFINITY, f64::INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);
                    for &(px, py) in &g.pts {
                        b0 = b0.min(px);
                        b1 = b1.min(py);
                        b2 = b2.max(px);
                        b3 = b3.max(py);
                    }
                    [b0 - 0.045, b1 - 0.045, b2 + 0.045, b3 + 0.065]
                })
                .collect();
            // the light falls from above and a little to one side
            let (lx, ly) = (0.35, -0.94);
            let nn = cw as usize;
            let mut img = ctx.create_image_data(cw, ch);
            par_rows(&mut img.data, nn * 4, |j, out| {
                let y = (j as f64 + 0.5) * s;
                for i in 0..nn {
                    let x = (i as f64 + 0.5) * s;
                    let mut px = Px::CLEAR;
                    // the nearest gouge: how far across it (and to which side),
                    // how wide and deep it is there, its heading, how far along
                    let mut best = (f64::INFINITY, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
                    let mut dust_v = 0.0f64;
                    for (g, bb) in gouges.iter().zip(&bounds) {
                        if x < bb[0] || y < bb[1] || x > bb[2] || y > bb[3] {
                            continue;
                        }
                        for k in 0..g.pts.len() - 1 {
                            let (a, b) = (g.pts[k], g.pts[k + 1]);
                            if (x - a.0).abs() > 0.04 || (y - a.1).abs() > 0.06 {
                                continue;
                            }
                            let (ex, ey) = (b.0 - a.0, b.1 - a.1);
                            let l2 = ex * ex + ey * ey;
                            let t = (((x - a.0) * ex + (y - a.1) * ey) / l2).clamp(0.0, 1.0);
                            let (qx, qy) = (a.0 + ex * t, a.1 + ey * t);
                            let d = (x - qx).hypot(y - qy);
                            let hw = g.half[k] + (g.half[k + 1] - g.half[k]) * t;
                            let dp = g.depth[k] + (g.depth[k + 1] - g.depth[k]) * t;
                            let side = if (x - qx) * ey - (y - qy) * ex > 0.0 { 1.0 } else { -1.0 };
                            if d - hw < best.0 - best.1 {
                                let l = l2.sqrt();
                                best = (d, hw, dp, side, ex / l, ey / l, (k as f64 + t) * l);
                            }
                            // the dust that fell: under the gouge, straight down the wall
                            if y > qy && (x - qx).abs() < hw * 2.0 + 0.002 {
                                let fall = y - qy;
                                dust_v = dust_v.max(dp * (-fall / 0.035).exp() * sstep(hw * 2.0 + 0.002, 0.0, (x - qx).abs()));
                            }
                        }
                    }
                    let (d, hw, dp, side, tx, ty, along) = best;
                    // gypsum dust fallen below, streaked by the wall's grain
                    if dust_v > 0.0 {
                        let streak = 0.4 + 0.6 * vnoise(seed ^ 0x2d, x * 900.0, y * 50.0);
                        px.lay(dust, dust_v * streak * 0.4);
                    }
                    if hw > 2e-4 {
                        // a ragged edge: the surface torn, not cut
                        let rag = (fbm(seed ^ 0x11, x * 600.0, y * 600.0, 2) - 0.5) * (hw * 0.7 + 0.001);
                        let e = d - hw - rag;
                        // the light across the gouge: + from its +n side (n = (ty, -tx))
                        let nl = ty * lx - tx * ly;
                        // the torn surface round it: paint and paper lifted and
                        // split back from the tear, pale where its core shows
                        let fringe = hw * 0.55 + 0.0015 * (0.5 + vnoise(seed ^ 0x19, along * 120.0, side));
                        if e > 0.0 && e < fringe && surface != 2 {
                            let k = sstep(fringe, fringe * 0.5, e) * (0.55 + 0.45 * vnoise(seed ^ 0x1a, x * 1500.0, y * 1500.0));
                            px.lay(edge, k * 0.85 * dp.sqrt().min(1.0));
                        }
                        if surface == 2 && e > 0.0 && e < 0.0015 {
                            // flakes of glaze chipped out along it
                            let chip = (hash01(seed ^ 0x33, (x / 0.0012) as i32, (y / 0.0012) as i32) > 0.55) as u8 as f64;
                            px.lay(edge, chip * 0.7 * dp);
                        }
                        // the lifted lip's shadow, on the side away from the light
                        if e > fringe * 0.4 && side * nl < 0.0 {
                            px.lay([0.07, 0.06, 0.05], 0.5 * dp * (-nl * side).min(1.0) * (-(e - fringe * 0.4) / 0.0012).exp());
                        }
                        if e <= aa {
                            // inside: the core, scored along by the nail; the wall
                            // facing the light lit, the other in the lip's shadow,
                            // darker down its middle where it is deepest
                            let u = (d / (hw + rag).max(1e-4)).min(1.0);
                            let lit = -side * nl * u;
                            // the lip's shadow reaches only a little way in
                            let shade = if lit < 0.0 { 1.0 + 0.55 * lit * sstep(0.45, 0.9, u) } else { 1.0 + 0.15 * lit };
                            let deepest = 1.0 - 0.18 * dp * (1.0 - u * u);
                            let scored = 0.9 + 0.18 * vnoise(seed ^ 0x44, (d * side) * 2500.0, along * 30.0);
                            let grime = 0.9 + 0.1 * fbm(seed ^ 0x45, x * 300.0, y * 300.0, 2);
                            let k = (shade * deepest * scored * grime).clamp(0.35, 1.15);
                            // the brown plies of the face paper at the tear's edge
                            let base = if surface == 0 && u > 0.78 { plies } else { core };
                            let c = [base[0] * k, base[1] * k, base[2] * k];
                            px.lay(c, sstep(aa, -aa, e) * (0.7 + 0.3 * dp.sqrt()));
                        }
                    }
                    // a strip of paper rolled up from the tear
                    for &(cx, cy, a, l) in &curls {
                        let (ux, uy) = ((x - cx) * a.cos() + (y - cy) * a.sin(), -(x - cx) * a.sin() + (y - cy) * a.cos());
                        let q = (ux / l).hypot(uy / (l * 0.3));
                        if q < 1.6 {
                            // its shadow on the wall beside it, then the curl, lit on top
                            let sq = ((ux - 0.0006) / l).hypot((uy - 0.0011) / (l * 0.34));
                            px.lay([0.06, 0.05, 0.04], 0.18 * sstep(1.15, 0.85, sq));
                            let shade = 0.82 + 0.25 * (-uy / (l * 0.3)).clamp(-1.0, 1.0);
                            px.lay([edge[0] * shade, edge[1] * shade, edge[2] * shade], sstep(1.0, 0.85, q));
                        }
                    }
                    // nothing at the plane's edges
                    px.a *= sstep(0.0, 0.01, x.min(pw - x).min(y).min(ph - y));
                    let c = px.rgba();
                    for (k, v) in c.iter().enumerate() {
                        out[i * 4 + k] = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
                    }
                }
            });
            ctx.put_image_data(&img, 0, 0);
        }
    });
    mark_mat(&tex)
}

/// A number that differs from one placing to the next (from where it was
/// put), to vary what its painter draws: a painter's own stream starts the
/// same on every floor
fn vary_by(a: i32, b: i32, c: i32) -> u32 {
    (hash01(0x7a3c_51d9 ^ c as u32, a, b) * 4.0e9) as u32
}

// Chalk arrow: 5% chance per level. A way-mark someone left on a wall at
// eye height (`assets/src/blender/props/r11a.notes.md`): an arrow drawn
// quickly in blackboard chalk, its head a closed triangle scribbled in (never
// her open arrow with the longer left barb, docs/SECRETS.md), a word over it
// in capitals in a hurried hand; the grain of the chalk caught on the wall's
// tooth, the strokes broken where it skipped, its dust under them; white
// chalk gone a little grey on the wall. Painted on a transparent plane the
// size of the old one, facing the room.
pub(crate) fn place_chalk_arrows(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    // the old flip (the plane turned to the wall): now which way it points
    let left = rnd() < 0.5;
    let tex = chalk_arrow_tex(left, vary_by(c.x * 4 + c.dx, c.z * 4 + c.dz, 0x513));
    let mat = lambert!({
        map: &tex,
        color: CHALK_ARROW_WHITE,
        transparent: true,
        depthWrite: false,
        polygonOffset: true,
        polygonOffsetFactor: -2,
        polygonOffsetUnits: -2
    });
    let decal = mesh(plane_geo!(0.50, 0.25), &mat);
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.008);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.008);
    let center_y = 1.55;
    if c.dx.abs() == 1 {
        decal.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        decal.position().set(wall_x, center_y, wall_z + (rnd() - 0.5) * 0.6);
    } else {
        decal.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        decal.position().set(wall_x + (rnd() - 0.5) * 0.6, center_y, wall_z);
    }
    g.world.add(&decal);
}

/// Blackboard chalk on a wall, years on (sRGB)
const CHALK_ARROW_WHITE: u32 = 0xdcd8ce;

/// A stroke of chalk: its points (px) and how hard it was pressed at each
struct ChalkLine {
    pts: Vec<(f64, f64)>,
    press: Vec<f64>,
}

/// A chalk arrow and its word on a plane 0.50 × 0.25 m (1.3 mm a px),
/// pointing left or right; white (the material tints it)
fn chalk_arrow_tex(left: bool, vary: u32) -> Tex {
    mark_tex(384.0, 192.0, [1.0, 1.0, 1.0], vary, move |ctx, w, h| {
        let seed = (rnd() * 4.0e9) as u32;
        let px_m = 0.5 / w;
        let mut lines: Vec<ChalkLine> = vec![];
        let jig = |p: (f64, f64), k: f64| (p.0 + (rnd() - 0.5) * k, p.1 + (rnd() - 0.5) * k);
        // a pressure along a stroke (its points every 3 px or less): heavier in
        // its middle, lifting at its ends
        let pressed = |n: usize, base: f64| -> Vec<f64> {
            let wob = rnd() * 6.0;
            (0..n)
                .map(|k| {
                    let t = k as f64 / (n.max(2) - 1) as f64;
                    base * (0.75 + 0.2 * (t * 7.0 + wob).sin()) * (t / 0.08).min(1.0).max(0.35) * ((1.0 - t) / 0.12).min(1.0).max(0.25)
                })
                .collect()
        };
        // the arrow, drawn rightward then mirrored if it points left: a shaft
        // 26-34 cm, a closed head 5 cm long and 5 cm across, filled by scribbling
        let shaft_y = h * (0.66 + 0.06 * rnd());
        let x0 = w * (0.1 + 0.05 * rnd());
        let x1 = w * (0.8 + 0.04 * rnd());
        let sag = (rnd() - 0.5) * 8.0;
        let tilt = (rnd() - 0.5) * 10.0;
        let shaft: Vec<(f64, f64)> = (0..=8)
            .map(|k| k as f64 / 8.0)
            .map(|t| jig((x0 + (x1 - x0) * t, shaft_y + tilt * (t - 0.5) + sag * 4.0 * t * (1.0 - t)), 1.5))
            .collect();
        let tip_y = shaft_y + tilt * 0.5;
        let (hl, hh) = (0.05 / px_m * (0.9 + 0.2 * rnd()), 0.025 / px_m * (0.9 + 0.2 * rnd()));
        let tip = (x1 + hl * 0.55, tip_y);
        let (b0, b1) = ((x1 - hl * 0.45, tip_y - hh), (x1 - hl * 0.45, tip_y + hh));
        let shaft = chalk_dense(&chalk_smooth(&shaft), 3.0);
        lines.push(ChalkLine { press: pressed(shaft.len(), 1.0), pts: shaft });
        // the head's outline in one go, then scribbled in, back and forth
        let outline = chalk_dense(&[jig(b0, 2.0), jig(tip, 1.5), jig(b1, 2.0), jig(b0, 3.0)], 3.0);
        lines.push(ChalkLine { press: pressed(outline.len(), 1.0), pts: outline });
        let mut fill = vec![];
        let rows = 7 + (rnd() * 3.0) as usize;
        for k in 0..=rows {
            let f = k as f64 / rows as f64;
            let y = b0.1 + (b1.1 - b0.1) * f;
            // the head's width at this height
            let reach = (1.0 - (2.0 * f - 1.0).abs()) * (tip.0 - b0.0);
            let a = (b0.0 + 2.0 + rnd() * 2.0, y + (rnd() - 0.5) * 2.0);
            let b = (b0.0 + reach - 3.0 - rnd() * 3.0, y + (rnd() - 0.5) * 2.0);
            if k % 2 == 0 {
                fill.push(a);
                fill.push(b);
            } else {
                fill.push(b);
                fill.push(a);
            }
        }
        let fill = chalk_dense(&fill, 3.0);
        lines.push(ChalkLine { press: pressed(fill.len(), 0.8), pts: fill });
        if left {
            for l in &mut lines {
                for p in &mut l.pts {
                    p.0 = w - p.0;
                }
            }
        }
        // the word over it, in capitals 5 cm tall, slanting and drifting
        const WORDS: [&str; 5] = ["THIS WAY", "RUN", "HELP", "HIDE", "NO"];
        let word = WORDS[(rnd() * WORDS.len() as f64) as usize % WORDS.len()];
        let lh = 0.05 / px_m * (0.9 + 0.2 * rnd());
        let slant = 0.05 + 0.12 * rnd();
        let width: f64 = word.chars().map(|c| if c == ' ' { 0.55 } else { chalk_letter_w(c) + 0.28 } * lh).sum::<f64>() - 0.28 * lh;
        let mut cx = (w - width) / 2.0 + (rnd() - 0.5) * 20.0;
        let base = shaft_y - h * 0.18 - (rnd() * 8.0);
        let drift = (rnd() - 0.5) * 0.08;
        for ch in word.chars() {
            if ch == ' ' {
                cx += 0.55 * lh;
                continue;
            }
            let lw = chalk_letter_w(ch) * lh;
            let y0 = base - lh + drift * (cx - w / 2.0) + (rnd() - 0.5) * 3.0;
            let sz = 0.94 + 0.12 * rnd();
            for st in chalk_letter(ch) {
                let pts: Vec<(f64, f64)> =
                    st.iter().map(|&(u, v)| jig((cx + u * lw * sz + (1.0 - v) * lh * slant, y0 + v * lh * sz), 2.2)).collect();
                let pts = chalk_dense(&chalk_smooth(&pts), 3.0);
                lines.push(ChalkLine { press: pressed(pts.len(), 0.9), pts });
            }
            cx += lw + 0.28 * lh;
        }
        let alpha = chalk_raster(&lines, w, h, 6.5, seed);
        let mut img = ctx.create_image_data(w, h);
        for (k, a) in alpha.iter().enumerate() {
            img.data[k * 4..k * 4 + 4].copy_from_slice(&[255, 255, 255, (a.clamp(0.0, 1.0) * 255.0).round() as u8]);
        }
        ctx.put_image_data(&img, 0, 0);
    })
}

/// A letter's width for its height (capitals in a quick hand)
fn chalk_letter_w(c: char) -> f64 {
    match c {
        'I' => 0.12,
        'W' => 0.85,
        'L' | 'E' | 'P' | 'S' => 0.5,
        _ => 0.62,
    }
}

/// The strokes of a capital in its box (u across, v down, 0..1), as a quick
/// hand draws it
fn chalk_letter(c: char) -> Vec<Vec<(f64, f64)>> {
    let arc = |cx: f64, cy: f64, rx: f64, ry: f64, a0: f64, a1: f64| -> Vec<(f64, f64)> {
        (0..=12).map(|k| a0 + (a1 - a0) * k as f64 / 12.0).map(|a| (cx + rx * a.cos(), cy + ry * a.sin())).collect()
    };
    match c {
        'A' => vec![vec![(0.0, 1.0), (0.5, 0.0), (1.0, 1.0)], vec![(0.22, 0.62), (0.8, 0.6)]],
        'D' => {
            let mut s = vec![(0.08, 1.0), (0.08, 0.0)];
            s.extend(arc(0.2, 0.5, 0.75, 0.5, -1.57, 1.57));
            s.push((0.06, 1.0));
            vec![s]
        }
        'E' => vec![vec![(0.95, 0.0), (0.05, 0.0), (0.05, 1.0), (1.0, 1.0)], vec![(0.05, 0.5), (0.75, 0.48)]],
        'H' => vec![vec![(0.05, 0.0), (0.05, 1.0)], vec![(0.95, 0.0), (0.95, 1.0)], vec![(0.05, 0.52), (0.95, 0.5)]],
        'I' => vec![vec![(0.5, 0.0), (0.5, 1.0)]],
        'L' => vec![vec![(0.05, 0.0), (0.05, 1.0), (0.95, 1.0)]],
        'N' => vec![vec![(0.05, 1.0), (0.05, 0.0), (0.95, 1.0), (0.95, 0.0)]],
        'O' => vec![arc(0.5, 0.5, 0.5, 0.5, -1.8, 4.6)],
        'P' => {
            let mut s = vec![(0.05, 1.0), (0.05, 0.0)];
            s.extend(arc(0.3, 0.27, 0.65, 0.27, -1.57, 1.57));
            s.push((0.05, 0.54));
            vec![s]
        }
        'R' => {
            let mut s = vec![(0.05, 1.0), (0.05, 0.0)];
            s.extend(arc(0.3, 0.26, 0.65, 0.26, -1.57, 1.57));
            s.extend([(0.05, 0.52), (0.4, 0.52), (0.98, 1.0)]);
            vec![s]
        }
        'S' => {
            let mut s = arc(0.5, 0.25, 0.48, 0.25, -0.3, -4.4);
            s.extend(arc(0.5, 0.74, 0.5, 0.26, -1.6, 2.6));
            vec![s]
        }
        'T' => vec![vec![(0.0, 0.0), (1.0, 0.02)], vec![(0.5, 0.0), (0.5, 1.0)]],
        'U' => {
            let mut s = vec![(0.05, 0.0)];
            s.extend(arc(0.5, 0.55, 0.45, 0.45, 3.14, 0.0));
            s.push((0.95, 0.0));
            vec![s]
        }
        'W' => vec![vec![(0.0, 0.0), (0.24, 1.0), (0.5, 0.3), (0.76, 1.0), (1.0, 0.0)]],
        'Y' => vec![vec![(0.0, 0.0), (0.5, 0.5), (1.0, 0.0)], vec![(0.5, 0.5), (0.5, 1.0)]],
        _ => vec![],
    }
}

/// A hand's line through the points (Chaikin, twice)
fn chalk_smooth(p: &[(f64, f64)]) -> Vec<(f64, f64)> {
    let mut p = p.to_vec();
    for _ in 0..2 {
        if p.len() < 3 {
            break;
        }
        let mut q = vec![p[0]];
        for k in 0..p.len() - 1 {
            let (a, b) = (p[k], p[k + 1]);
            q.push((a.0 * 0.75 + b.0 * 0.25, a.1 * 0.75 + b.1 * 0.25));
            q.push((a.0 * 0.25 + b.0 * 0.75, a.1 * 0.25 + b.1 * 0.75));
        }
        q.push(p[p.len() - 1]);
        p = q;
    }
    p
}

/// The side of the cells a chalk raster sorts its strokes' segments into (px)
const CHALK_CELL: usize = 16;

/// For each `CHALK_CELL` square of an `n × m` px canvas, the segments (their
/// index in `segs`) that come within `reach` px of it
fn chalk_cells(segs: &[((f64, f64), (f64, f64), f64, f64, u32)], n: usize, m: usize, reach: f64) -> Vec<Vec<u32>> {
    let (cw, chh) = (n.div_ceil(CHALK_CELL), m.div_ceil(CHALK_CELL));
    let mut cells = vec![vec![]; cw * chh];
    let cell = |v: f64, hi: usize| ((v / CHALK_CELL as f64).floor().max(0.0) as usize).min(hi - 1);
    for (k, &(a, b, ..)) in segs.iter().enumerate() {
        let (x0, x1) = (cell(a.0.min(b.0) - reach, cw), cell(a.0.max(b.0) + reach, cw));
        let (y0, y1) = (cell(a.1.min(b.1) - reach, chh), cell(a.1.max(b.1) + reach, chh));
        for cy in y0..=y1 {
            for cx in x0..=x1 {
                cells[cy * cw + cx].push(k as u32);
            }
        }
    }
    cells
}

/// The polyline `p` with points added so no step is longer than `step`
fn chalk_dense(p: &[(f64, f64)], step: f64) -> Vec<(f64, f64)> {
    let mut out = vec![];
    for k in 0..p.len().saturating_sub(1) {
        let (a, b) = (p[k], p[k + 1]);
        let n = ((a.0 - b.0).hypot(a.1 - b.1) / step).ceil().max(1.0) as usize;
        for i in 0..n {
            let t = i as f64 / n as f64;
            out.push((a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t));
        }
    }
    if let Some(&l) = p.last() {
        out.push(l);
    }
    out
}

/// The chalk left on a wall by `lines`, a stick `width` px wide: how much each
/// pixel of a `w × h` canvas holds (0..1). The chalk catches on the wall's
/// tooth (fixed grains about a millimetre across, coarser patches of the
/// wall's texture), heavier where it was pressed, gone where it skipped; its
/// dust has fallen down the wall under the strokes. No draws.
fn chalk_raster(lines: &[ChalkLine], w: f64, h: f64, width: f64, seed: u32) -> Vec<f64> {
    let (n, m) = (w as usize, h as usize);
    // each segment with its bounds, so a pixel looks only at those near it
    let mut segs: Vec<((f64, f64), (f64, f64), f64, f64, u32)> = vec![];
    for (li, l) in lines.iter().enumerate() {
        for k in 0..l.pts.len().saturating_sub(1) {
            segs.push((l.pts[k], l.pts[k + 1], l.press[k], l.press[k + 1], seed.wrapping_add(li as u32 * 97)));
        }
    }
    let r = width / 2.0;
    let cells = chalk_cells(&segs, n, m, r + 1.0);
    let cw = n.div_ceil(CHALK_CELL);
    let mut dep = vec![0.0f64; n * m];
    let mut bytes = vec![0u8; n * m * 4];
    par_rows(&mut bytes, n * 4, |j, out| {
        let y = j as f64 + 0.5;
        for i in 0..n {
            let x = i as f64 + 0.5;
            let mut keep = 0.0f64;
            for &si in &cells[(j / CHALK_CELL) * cw + i / CHALK_CELL] {
                let (a, b, pa, pb, ls) = segs[si as usize];
                if x < a.0.min(b.0) - r - 1.0 || x > a.0.max(b.0) + r + 1.0 || y < a.1.min(b.1) - r - 1.0 || y > a.1.max(b.1) + r + 1.0 {
                    continue;
                }
                let (ex, ey) = (b.0 - a.0, b.1 - a.1);
                let l2 = (ex * ex + ey * ey).max(1e-9);
                let t = (((x - a.0) * ex + (y - a.1) * ey) / l2).clamp(0.0, 1.0);
                let d = (x - a.0 - ex * t).hypot(y - a.1 - ey * t);
                // the stick's edge is uneven; it skips along the stroke
                let rr = r * (0.85 + 0.3 * vnoise(ls ^ 0x5a, x * 0.35, y * 0.35));
                let cover = sstep(rr + 0.8, rr - 0.8, d);
                if cover <= 0.0 {
                    continue;
                }
                let skip = sstep(0.2, 0.5, vnoise(ls ^ 0x77, (a.0 + ex * t) * 0.11, (a.1 + ey * t) * 0.11));
                let press = (pa + (pb - pa) * t) * (0.3 + 0.7 * skip) * (1.0 - 0.45 * (d / rr).powi(2));
                keep = 1.0 - (1.0 - keep) * (1.0 - cover * press);
            }
            // the wall's tooth: grains ~1.3 px, and coarser patches of it
            let grain = hash01(seed ^ 0x1d, (x / 1.3) as i32, (y / 1.3) as i32);
            let patch = vnoise(seed ^ 0x2e, x * 0.12, y * 0.12);
            let tooth = 0.7 * grain + 0.3 * patch;
            // chalk on a wall holds only on the tooth's peaks
            let a = sstep(tooth - 0.06, tooth + 0.05, keep * 0.82);
            out[i * 4] = (a * 255.0).round() as u8;
            out[i * 4 + 1] = (keep * 255.0).round() as u8;
        }
    });
    for k in 0..n * m {
        dep[k] = f64::from(bytes[k * 4]) / 255.0;
    }
    // the dust: what fell from each stroke, down the wall under it, fading
    let mut out = dep.clone();
    for i in 0..n {
        let mut acc = 0.0f64;
        for j in 0..m {
            let k = j * n + i;
            let held = f64::from(bytes[k * 4 + 1]) / 255.0;
            acc = acc * 0.93 + held * 0.1;
            let speck = hash01(seed ^ 0x3f, i as i32, j as i32);
            let dust = acc * (0.12 + 0.5 * (speck > 0.93) as u8 as f64);
            out[k] = 1.0 - (1.0 - out[k]) * (1.0 - dust.min(0.5));
        }
    }
    out
}

// Dust bunnies: 10% chance per level. Where no air moves, what a floor
// sheds (textile lint, hair, skin, grit) felts itself into clumps held by
// static and their own fibres (`assets/src/blender/props/r11a.notes.md`):
// five to ten of them, from a small wisp to a sprawl of fluff 6 cm long
// (batch r11a), grey with a fleck of a jumper's colour here and there, on a
// thin film of the dust they gather from and the loose fibres and hairs
// round them.
pub(crate) fn place_dust_bunnies(g: &mut Game, _dist: &[i32]) {
    use game::assets::props::prop_part_as_modelled;
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let hard = puddle_floor_kind(g, x, z) != 0;
    let grp = group();
    let fluff = dust_fluff_tex();
    let fringe = dust_fringe_tex();
    let n_bunnies = 5 + (rnd() * 6.0).floor() as i32;
    let mut spots = vec![];
    for i in 0..n_bunnies {
        // the old ball's draws: its colour (which bunny), its radius (how big,
        // within 13%), its three scales (its turn, its grey, and one kept) and
        // where it lay
        let kind = ((rnd() * 3.0).floor() as usize).min(2);
        let k = 0.87 + (rnd() * 0.006) / 0.006 * 0.26;
        let yaw = rnd() * PI * 2.0;
        let grey = rnd();
        let _ = rnd(); // keeps the old draw (its depth's scale)
        let (bx, bz) = ((rnd() - 0.5) * 0.40, (rnd() - 0.5) * 0.40);
        let v = 0.84 + 0.16 * grey;
        let byte = |f: f64| ((f * 255.0).round() as u32).min(255);
        let tint = byte(v) << 16 | byte(v) << 8 | byte(v * 0.99);
        let bunny = group();
        let tag = ["A", "B", "C"][kind];
        if let Some(geo) = prop_part_as_modelled(&format!("Real__dustBunny{tag}")) {
            bunny.add(&mesh(geo, &lambert!({ map: &fluff, color: tint })));
        }
        // the loose fuzz round it: its fibres cut out of a shell
        if let Some(geo) = prop_part_as_modelled(&format!("Real__dustBunnyFuzz{tag}")) {
            bunny.add(&mesh(geo, &lambert!({ map: &fringe, color: tint, alphaTest: 0.5, side: DoubleSide })));
        }
        bunny.scale().set_scalar(k);
        bunny.rotation().y = yaw;
        bunny.position().set(bx, 0.0, bz);
        grp.add(&bunny);
        spots.push((bx, bz, 0.03 * k));
    }
    // the film of dust they lie on, and its loose fibres
    let film = mesh(
        {
            let geo = plane_geo!(0.46, 0.46);
            geo.rotate_x(-PI / 2.0);
            geo
        },
        &mark_mat(&dust_film_tex(spots, hard, vary_by(x, z, 0x510))),
    );
    film.position().y = 0.0015;
    grp.add(&film);
    scatter_on_floor(g, &grp, x, z, 2.4);
    g.world.add(&grp);
}
