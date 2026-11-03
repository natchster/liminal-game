//! The placers of this crate, in registration order.
use crate::textures::*;
use crate::{frame_bars, mount_on_wall, open_tiles, scatter_on_floor, tri_geo, wall_candidates, WallCand};
use game::prelude::*;

/// A uniformly random element (one `rnd()` draw). for colour tables.
fn pick_hex(a: &[u32]) -> u32 {
    a[(rnd() * a.len() as f64).floor() as usize]
}

// R521 hospital cross: 4% chance per level. Red + cross box wall-mounted.
pub(crate) fn place_hospital_crosses(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    let white_mat = basic_mat!({ color: 0xf0ece0 });
    let red_mat = basic_mat!({ color: 0xd82030 });
    let cross = group();
    let s = 0.14;
    // White backing.
    let back = mesh(pg("HospitalCrosses__back", box_geo!(s, s, 0.010)), &white_mat);
    back.position().z = -0.005;
    cross.add(&back);
    // Red vertical bar.
    let v_bar = mesh(pg("HospitalCrosses__vbar", box_geo!(s * 0.3, s * 0.9, 0.008)), &red_mat);
    v_bar.position().z = 0.002;
    cross.add(&v_bar);
    // Red horizontal bar.
    let h_bar = mesh(pg("HospitalCrosses__hbar", box_geo!(s * 0.9, s * 0.3, 0.008)), &red_mat);
    h_bar.position().z = 0.002;
    cross.add(&h_bar);
    mount_on_wall(g, &cross, &c, 0.005, 2.0, 0.6);
    g.world.add(&cross);
}

// R521 children's tea set: 5% chance per level. 3-4 mini cups + teapot.
pub(crate) fn place_childrens_tea_sets(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let set_colors = [0xf0c0d0, 0xd0d8f0, 0xf0e8d8, 0xd0f0d8];
    let set_mat = lambert!({ color: pick_hex(&set_colors) });
    let tea_mat = lambert!({ color: 0x603820 });
    let grp = group();
    // Teapot in center: sphere + spout + handle + lid.
    let pot = mesh(pg("ChildrensTeaSets__pot", sphere_geo!(0.024, 10, 8)), &set_mat);
    pot.scale().y = 0.8;
    pot.position().set(0, 0.019, 0);
    grp.add(&pot);
    let spout = mesh(pg("ChildrensTeaSets__spout", cyl_geo!(0.005, 0.003, 0.020, 5)), &set_mat);
    spout.rotation().z = -0.5;
    spout.position().set(0.024, 0.024, 0);
    grp.add(&spout);
    let handle = mesh(pg("ChildrensTeaSets__handle", torus_geo!(0.008, 0.002, 4, 8, PI)), &set_mat);
    handle.rotation().z = PI / 2.0; // C in the XY plane, bulging out behind (-X)
    handle.position().set(-0.0215, 0.020, 0); // both ends in the pot wall
    grp.add(&handle);
    let lid = mesh(pg("ChildrensTeaSets__lid", sphere_geo!(0.006, 6, 5)), &set_mat);
    lid.position().set(0, 0.038, 0);
    grp.add(&lid);
    // 3-4 mini cups around.
    let n_cups = 3 + (rnd() * 2.0).floor() as i32;
    for i in 0..n_cups {
        let a = (i as f64 / n_cups as f64) * PI * 2.0;
        let cup = mesh(pg("ChildrensTeaSets__cup", cyl_geo!(0.011, 0.008, 0.015, 8, 1, true)), &set_mat);
        cup.position().set(a.cos() * 0.06, 0.008, a.sin() * 0.06);
        grp.add(&cup);
        // Saucer.
        let saucer = mesh(pg("ChildrensTeaSets__saucer", cyl_geo!(0.018, 0.018, 0.002, 10)), &set_mat);
        saucer.position().set(a.cos() * 0.06, 0.001, a.sin() * 0.06);
        grp.add(&saucer);
        // Tea residue.
        if rnd() < 0.5 {
            let tea = mesh(circle_geo!(0.008, 8), &tea_mat);
            tea.rotation().x = -PI / 2.0;
            tea.position().set(a.cos() * 0.06, 0.013, a.sin() * 0.06);
            grp.add(&tea);
        }
    }
    scatter_on_floor(g, &grp, x, z, 2.0);
    g.world.add(&grp);
}

// R520 coin scatter: 10% chance per level. 5-12 loose coins on floor.
pub(crate) fn place_coin_scatters(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let coin_colors = [0xd8b040, 0xa08050, 0xc0c0c0, 0xe0a060];
    let grp = group();
    let n_coins = 5 + (rnd() * 8.0).floor() as i32;
    for i in 0..n_coins {
        let coin_mat = lambert!({ color: pick_hex(&coin_colors) });
        let coin = mesh(pg("CoinScatters__coin", cyl_geo!(0.008, 0.008, 0.001, 10)), &coin_mat);
        coin.position().set((rnd() - 0.5) * 0.30, 0.001, (rnd() - 0.5) * 0.30);
        coin.rotation().y = rnd() * PI * 2.0;
        // 20% chance edge-on tilted.
        if rnd() < 0.20 {
            coin.rotation().z = PI / 2.0;
            coin.position().y = 0.008;
        }
        grp.add(&coin);
    }
    scatter_on_floor(g, &grp, x, z, 2.4);
    g.world.add(&grp);
}

// R519 wedding ring: 6% chance per level. Small gold torus on floor.
pub(crate) fn place_wedding_rings(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let gold_colors = [0xd8b040, 0xe8c860, 0xd0d0d0, 0xf0d8d8];
    let gold_mat = lambert!({ color: pick_hex(&gold_colors) });
    let gem_mat = basic_mat!({ color: 0xe0f0ff, transparent: true, opacity: 0.9 });
    let ring = group();
    // Band torus.
    let band = mesh(pg("WeddingRings__band", torus_geo!(0.010, 0.002, 5, 14)), &gold_mat);
    band.rotation().x = PI / 2.0;
    band.position().y = 0.002;
    ring.add(&band);
    // 40% chance small diamond gem on top (an octahedron).
    if rnd() < 0.40 {
        let gem = mesh(octa_geo!(0.003), &gem_mat);
        gem.position().y = 0.006;
        ring.add(&gem);
    }
    scatter_on_floor(g, &ring, x, z, 2.4);
    g.world.add(&ring);
}

// R519 water puddle: 8% chance per level. Where a drip from the ceiling
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
    g.pp.puddle_positions.clear(); // R520 reset per level
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
    // R520 track for drip audio.
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

// ------------------------------------------------------------ painting marks
// (the helpers the painted marks of this crate share: noise, laying colour,
// a mark's texture and material; none of them draws)

/// A hash of (seed, i, j) in [0, 1)
fn hash01(seed: u32, i: i32, j: i32) -> f64 {
    let mut h = seed ^ (i as u32).wrapping_mul(0x27d4_eb2d) ^ (j as u32).wrapping_mul(0x1656_67b1);
    h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
    h = (h ^ (h >> 13)).wrapping_mul(0xc2b2_ae35);
    h ^= h >> 16;
    f64::from(h) / 4_294_967_296.0
}

/// Smooth value noise in [0, 1]
fn vnoise(seed: u32, x: f64, y: f64) -> f64 {
    let (xf, yf) = (x.floor(), y.floor());
    let (i, j) = (xf as i32, yf as i32);
    let (u, v) = (x - xf, y - yf);
    let (u, v) = (u * u * (3.0 - 2.0 * u), v * v * (3.0 - 2.0 * v));
    let (a, b) = (hash01(seed, i, j), hash01(seed, i + 1, j));
    let (c, d) = (hash01(seed, i, j + 1), hash01(seed, i + 1, j + 1));
    a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v
}

/// Fractal value noise of `oct` octaves, in [0, 1]
fn fbm(seed: u32, x: f64, y: f64, oct: u32) -> f64 {
    let (mut s, mut amp, mut f, mut n) = (0.0, 0.5, 1.0, 0.0);
    for k in 0..oct {
        s += amp * vnoise(seed.wrapping_add(k * 7919), x * f, y * f);
        n += amp;
        amp *= 0.5;
        f *= 2.03;
    }
    s / n
}

/// Smoothstep from `e0` to `e1` (either way round)
fn sstep(e0: f64, e1: f64, x: f64) -> f64 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// The smooth minimum of two distances, blended over `k`
fn smin(a: f64, b: f64, k: f64) -> f64 {
    let h = (0.5 + 0.5 * (b - a) / k).clamp(0.0, 1.0);
    b + (a - b) * h - k * h * (1.0 - h)
}

/// How a dried water mark shades at signed distance `d` (m, < 0 inside) from
/// one of its rims: sharp on its outside (over `aa`), what the water left
/// there fading inward over `ew`
fn tide(d: f64, aa: f64, ew: f64) -> f64 {
    (0.5 - d / aa).clamp(0.0, 1.0) * (d.min(0.0) / ew).exp()
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
fn mark_tex<F: FnOnce(&mut Canvas, f64, f64)>(w: f64, h: f64, bleed: [f64; 3], vary: u32, paint: F) -> Tex {
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

// R518 porcelain cup + saucer: 7% chance per level.
pub(crate) fn place_porcelain_cup_saucers(g: &mut Game, _dist: &[i32]) {
    g.pp.teacup_positions.clear(); // R525 reset per level
    if rnd() < 0.93 {
        return;
    } // 7% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let porcelain_colors = [0xe8dcc0, 0xf0d8c0, 0xd8d8e0, 0xe8c0a0];
    let porcelain_mat = lambert!({ color: pick_hex(&porcelain_colors) });
    let tea_mat = lambert!({ color: 0x603820 });
    let pattern_mat = lambert!({ color: 0x4030a0 });
    let set = group();
    // Saucer (flat disc + slight rim).
    let saucer = mesh(pg("PorcelainCupSaucers__saucer", cyl_geo!(0.045, 0.045, 0.004, 14)), &porcelain_mat);
    saucer.position().y = 0.002;
    set.add(&saucer);
    let saucer_rim = mesh(pg("PorcelainCupSaucers__saucerRim", torus_geo!(0.043, 0.002, 5, 14)), &porcelain_mat);
    saucer_rim.rotation().x = PI / 2.0;
    saucer_rim.position().y = 0.005;
    set.add(&saucer_rim);
    // Cup (short cylinder).
    let cup = mesh(pg("PorcelainCupSaucers__cup", cyl_geo!(0.024, 0.020, 0.030, 12, 1, true)), &porcelain_mat);
    cup.position().y = 0.019;
    set.add(&cup);
    // Cup bottom.
    let cup_bottom = mesh(circle_geo!(0.020, 12), &porcelain_mat);
    cup_bottom.rotation().x = -PI / 2.0;
    cup_bottom.position().y = 0.005;
    set.add(&cup_bottom);
    // Cup handle (torus arc): loop in the radial plane with its gap on the cup wall.
    let handle = mesh(pg("PorcelainCupSaucers__handle", torus_geo!(0.010, 0.002, 4, 8, PI * 1.4)), &porcelain_mat);
    handle.rotation().z = -2.2;
    handle.rotation().y = 0.0;
    handle.position().set(0.028, 0.020, 0);
    set.add(&handle);
    // Tea residue (60% chance).
    if rnd() < 0.60 {
        let tea = mesh(circle_geo!(0.018, 12), &tea_mat);
        tea.rotation().x = -PI / 2.0;
        tea.position().y = 0.026;
        set.add(&tea);
    }
    // Small blue pattern band on cup (60% chance).
    if rnd() < 0.60 {
        let band = mesh(cyl_geo!(0.0245, 0.0245, 0.003, 12, 1, true), &pattern_mat);
        band.scale().set(0.935, 1, 0.935); // painted on the cup wall, not floating off it
        band.position().y = 0.010;
        set.add(&band);
    }
    scatter_on_floor(g, &set, x, z, 2.4);
    g.world.add(&set);
    // R525 track for clink audio.
    let p = set.pos();
    g.pp.teacup_positions.push(v3(p.x, 0.02, p.z));
}

// R518 broken alarm bell: 4% chance per level. Wall bell dome + dangling clapper.
pub(crate) fn place_alarm_bells(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    let plate_mat = lambert!({ color: 0x6a3820 });
    let bell_mat = lambert!({ color: 0xc02818 });
    let clapper_mat = lambert!({ color: 0x181410 });
    let bell = group();
    // Back plate.
    let plate = mesh(pg("AlarmBells__plate", box_geo!(0.12, 0.16, 0.014)), &plate_mat);
    plate.position().set(0, 0, -0.007);
    bell.add(&plate);
    // Bell dome (half-sphere).
    let dome = mesh(pg("AlarmBells__gong", sphere_geo!(0.055, 12, 8, 0, PI * 2.0, 0, PI / 2.0)), &bell_mat);
    dome.position().set(0, 0.020, 0.020);
    dome.rotation().x = PI / 2.0; // gong faces the room, apex at the nut
    bell.add(&dome);
    // Bell top small button.
    let top = mesh(pg("AlarmBells__nut", cyl_geo!(0.008, 0.008, 0.008, 6)), &bell_mat);
    top.rotation().x = PI / 2.0;
    top.position().set(0, 0.020, 0.075);
    bell.add(&top);
    // Dangling clapper (rusted, off-center).
    let clapper_pivot = group();
    clapper_pivot.position().set(0, 0.020, 0.080); // hangs from the centre nut
    let clapper_arm = mesh(pg("AlarmBells__arm", cyl_geo!(0.001, 0.001, 0.04, 3)), &clapper_mat);
    clapper_arm.position().y = -0.020;
    clapper_pivot.add(&clapper_arm);
    let ball = mesh(pg("AlarmBells__ball", sphere_geo!(0.006, 6, 5)), &clapper_mat);
    ball.position().y = -0.042;
    clapper_pivot.add(&ball);
    clapper_pivot.rotation().z = 0.4; // hanging tilted, broken
    bell.add(&clapper_pivot);
    mount_on_wall(g, &bell, &c, 0.007, 2.20, 0.6);
    g.world.add(&bell);
}

// R517 rust stain streak: 5% chance per level. Vertical rust drip decal on wall.
pub(crate) fn place_rust_stain_streaks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    let mat = basic_mat!({ map: rust_stain_texture(), transparent: true, opacity: 0.9, depthWrite: false });
    let decal = mesh(plane_geo!(0.24, 0.60), &mat);
    mount_on_wall(g, &decal, &c, 0.005, 1.5, 0.5);
    g.world.add(&decal);
}

// R517 forgotten wristwatch: 6% chance per level. Watch face + leather band on floor.
pub(crate) fn place_forgotten_wristwatches(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let band_colors = [0x2a1810, 0x603820, 0x404040, 0xd8b040];
    let band_mat = lambert!({ color: pick_hex(&band_colors) });
    let case_mat = lambert!({ color: 0xc89040 });
    let face_mat = lambert!({ map: watch_face_texture(), side: DoubleSide });
    let watch = group();
    // Case disc.
    let case_ = mesh(pg("ForgottenWristwatches__case", cyl_geo!(0.020, 0.020, 0.006, 12)), &case_mat);
    case_.position().y = 0.003;
    watch.add(&case_);
    // Face.
    let face = mesh(circle_geo!(0.018, 12), &face_mat);
    face.rotation().x = -PI / 2.0;
    face.position().y = 0.0065;
    watch.add(&face);
    // Band left half (curled); each strap starts at a lug on the case edge.
    let band_l = mesh(pg("ForgottenWristwatches__strapHoles", torus_geo!(0.030, 0.005, 4, 12, PI * 0.7)), &band_mat);
    band_l.rotation().x = -PI / 2.0;
    band_l.position().set(-0.030, 0.005, -0.020);
    watch.add(&band_l);
    // Band right half.
    let band_r = mesh(pg("ForgottenWristwatches__strapKeeper", torus_geo!(0.030, 0.005, 4, 12, PI * 0.7)), &band_mat);
    band_r.set_rotation_order(EulerOrder::YXZ); // lie flat first, then turn half round
    band_r.rotation().x = -PI / 2.0;
    band_r.rotation().y = PI;
    band_r.position().set(0.030, 0.005, 0.020);
    watch.add(&band_r);
    scatter_on_floor(g, &watch, x, z, 2.4);
    g.world.add(&watch);
}

// R516 wall claw marks: 4% chance per level. Something dragged its nails down
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

// R516 rusted nails: 10% chance per level. 6-12 rusted nails scattered on floor.
pub(crate) fn place_rusted_nails(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let rust_colors = [0x6a3820, 0x504028, 0x8a4030];
    let grp = group();
    let n_nails = 6 + (rnd() * 7.0).floor() as i32;
    for i in 0..n_nails {
        let nail_mat = lambert!({ color: pick_hex(&rust_colors) });
        let nail = group();
        // Shaft.
        let shaft = mesh(pg("RustedNails__shaft", cyl_geo!(0.001, 0.001, 0.032, 4)), &nail_mat);
        shaft.rotation().z = PI / 2.0;
        shaft.position().set(0.010, 0.001, 0);
        nail.add(&shaft);
        // Head.
        let head = mesh(pg("RustedNails__head", cyl_geo!(0.003, 0.003, 0.002, 6)), &nail_mat);
        head.rotation().z = PI / 2.0;
        head.position().set(-0.007, 0.001, 0);
        nail.add(&head);
        // Tip point.
        let tip = mesh(cone_geo!(0.001, 0.005, 4), &nail_mat);
        tip.rotation().z = -PI / 2.0;
        tip.position().set(0.028, 0.001, 0);
        nail.add(&tip);
        // rests on its head rim and point: tilt the shank down toward the tip
        nail.position().set((rnd() - 0.5) * 0.25, 0.0016, (rnd() - 0.5) * 0.20);
        nail.rotation().z = -0.055;
        nail.rotation().y = rnd() * PI * 2.0;
        grp.add(&nail);
    }
    scatter_on_floor(g, &grp, x, z, 2.4);
    g.world.add(&grp);
}

// R515 biohazard sign: 4% chance per level. An ISO 7010 biological hazard
// sign screwed to a wall (batch r13a).
pub(crate) fn place_biohazard_signs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    // An A4 rigid PVC safety sign (Blender batch `r13a`): the ISO 7010 W009
    // warning triangle, its trefoil drawn to Baldwin's construction, over
    // BIOHAZARD, screwed to the wall at its corners; its yellow faded, its
    // print scuffed to the white plastic, rust run from its screws, one in
    // three with a screw gone and that corner sprung off the wall
    // (docs/PROPS.md)
    use game::assets::props::prop_part_as_modelled;
    let sign = group();
    mount_on_wall(g, &sign, &c, 0.005, 1.70, 0.6);
    let p = sign.pos();
    let vary = bh_vary(p.x, p.z);
    let lifted = vary % 3 == 0;
    // the parts are modelled with the wall's face at z = 0: 5 mm behind the spot
    let body = group();
    body.position().z = -0.005;
    sign.add(&body);
    if let Some(geo) = prop_part_as_modelled(if lifted { "Real__biohazardPlateLifted" } else { "Real__biohazardPlate" }) {
        body.add(&mesh(geo, lambert!({ map: &biohazard_sign_tex(vary) })));
    }
    if let Some(geo) = prop_part_as_modelled(if lifted { "Real__biohazardScrews3" } else { "Real__biohazardScrews" }) {
        body.add(&mesh(geo, phong!({ color: 0x45403a, specular: 0x34322e, shininess: 28 })));
    }
    g.world.add(&sign);
}

/// Something of where a thing was put, to vary what its painters paint
/// (a painter's own stream starts the same on every floor)
fn bh_vary(x: f64, z: f64) -> u32 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for v in [x, z] {
        h ^= v.to_bits();
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    (h ^ (h >> 29) ^ (h >> 47)) as u32
}

/// Is the point (x, y), in the units of Baldwin and Runkle's construction
/// (the symbol's centre at the origin, y up; the outer circles 300 across),
/// inside the biohazard trefoil? Three circles of radius 150 whose centres
/// are 110 out at 90, 210 and 330 degrees, less three of radius 105 whose
/// centres are 150 out, less the centre's hole (60 across), the 10-wide
/// slits from the hole to each inner circle and the 40-wide gaps at each
/// lobe's end; and the ring (radius 100 to 135) where it crosses the inner
/// circles, 10 short of their edges
fn bh_trefoil(x: f64, y: f64) -> bool {
    let r = x.hypot(y);
    let mut outer = false;
    let mut inner = false;
    let mut ring = false;
    let mut cut = r < 30.0;
    for k in 0..3 {
        let a = PI / 2.0 + k as f64 * PI * 2.0 / 3.0;
        let (ux, uy) = (a.cos(), a.sin());
        if (x - ux * 110.0).hypot(y - uy * 110.0) < 150.0 {
            outer = true;
        }
        let di = (x - ux * 150.0).hypot(y - uy * 150.0);
        if di < 105.0 {
            inner = true;
        }
        if di < 95.0 && (100.0..=135.0).contains(&r) {
            ring = true;
        }
        let (along, across) = (x * ux + y * uy, (x * uy - y * ux).abs());
        if (0.0..=50.0).contains(&along) && across < 5.0 || (225.0..=275.0).contains(&along) && across < 20.0 {
            cut = true;
        }
    }
    outer && !inner && !cut || ring
}

/// The sign's face as its plate's front is mapped (210 x 297 mm): signal
/// yellow, the ISO 7010 W009 triangle (its black band 5.3% of its side, its
/// outer corners rounded) and the trefoil in it, BIOHAZARD under it in a
/// bold grotesque; faded by years of light, the print scuffed through to
/// the white PVC, grime, rust run down from the screws, dust on its top
fn biohazard_sign_tex(vary: u32) -> Tex {
    let ppm = 2000.0;
    canvas_tex_opt(420, 594, false, move |ctx, w, h| {
        let seed = (rnd() * 4.0e9) as u32 ^ vary;
        with_own_stream(seed, || {
            // faded signal yellow, the ink's black gone a little brown
            let fade = 0.25 + 0.2 * rnd();
            let yel = [234.0 + 8.0 * fade, 172.0 + 40.0 * fade, 30.0 + 90.0 * fade];
            let ink = [34.0 + 30.0 * fade, 30.0 + 26.0 * fade, 26.0 + 18.0 * fade];
            let rgb = |c: [f64; 3]| format!("rgb({:.0},{:.0},{:.0})", c[0], c[1], c[2]);
            ctx.set_fill_style(rgb(yel));
            ctx.fill_rect(0, 0, w, h);
            // the triangle: side 180 mm, its apex 12 mm from the top
            let s = 0.180 * ppm;
            let th = s * 3f64.sqrt() / 2.0;
            let (ax, ay) = (w / 2.0, 0.013 * ppm);
            let cr = s * 0.027;
            let tri = |ctx: &mut Canvas, k: f64| {
                // the triangle shrunk about its centroid by `k` px (perpendicular)
                let (gx, gy) = (ax, ay + th * 2.0 / 3.0);
                let f = 1.0 - k / (th / 3.0);
                let pts = [(ax, ay), (ax + s / 2.0, ay + th), (ax - s / 2.0, ay + th)];
                ctx.begin_path();
                for (i, &(x, y)) in pts.iter().enumerate() {
                    let (x, y) = (gx + (x - gx) * f, gy + (y - gy) * f);
                    if i == 0 {
                        ctx.move_to(x, y);
                    } else {
                        ctx.line_to(x, y);
                    }
                }
                ctx.close_path();
            };
            ctx.set_fill_style(rgb(ink));
            ctx.set_stroke_style(rgb(ink));
            ctx.set_line_join("round");
            ctx.set_line_width(cr * 2.0);
            tri(ctx, cr);
            ctx.fill();
            ctx.stroke();
            ctx.set_fill_style(rgb(yel));
            tri(ctx, s * 0.0533);
            ctx.fill();
            // BIOHAZARD
            ctx.set_fill_style(rgb(ink));
            ctx.set_text_align("center");
            ctx.set_font(&format!("bold {:.0}px Helvetica", 0.036 * ppm));
            ctx.fill_text_max("BIOHAZARD", w / 2.0, 0.232 * ppm, w - 0.03 * ppm);
            // the trefoil, each pixel tested against its construction (4 x 4 samples)
            let sym_h = th * 0.458;
            let u = sym_h / 465.0; // px a construction unit
            let (cx, cy) = (ax, ay + th * 0.665);
            let (x0, x1) = ((cx - 250.0 * u).floor().max(0.0) as usize, ((cx + 250.0 * u).ceil() as usize).min(w as usize));
            let (y0, y1) = ((cy - 265.0 * u).floor().max(0.0) as usize, ((cy + 210.0 * u).ceil() as usize).min(h as usize));
            let (bw, bh) = (x1 - x0, y1 - y0);
            let mut img = ctx.get_image_data(x0, y0, bw, bh);
            par_rows(&mut img.data, bw * 4, |j, out| {
                for i in 0..bw {
                    let mut n = 0;
                    for sj in 0..4 {
                        for si in 0..4 {
                            let px = (x0 + i) as f64 + (si as f64 + 0.5) / 4.0;
                            let py = (y0 + j) as f64 + (sj as f64 + 0.5) / 4.0;
                            if bh_trefoil((px - cx) / u, (cy - py) / u) {
                                n += 1;
                            }
                        }
                    }
                    let a = n as f64 / 16.0;
                    for k in 0..3 {
                        let v = f64::from(out[i * 4 + k]);
                        out[i * 4 + k] = (v + (ink[k] - v) * a).round() as u8;
                    }
                }
            });
            ctx.put_image_data(&img, x0, y0);
            // faded more where the light fell: toward its top
            let gf = ctx.create_linear_gradient(0, 0, 0, h);
            gf.add_color_stop(0, "rgba(246,236,200,0.16)");
            gf.add_color_stop(1, "rgba(246,236,200,0.04)");
            ctx.set_fill_style(&gf);
            ctx.fill_rect(0, 0, w, h);
            // grime: a film, heavier low down and round the edges, in patches
            for _ in 0..18 {
                let (x, y, r) = (rnd() * w, h * rnd().powf(0.6), 40.0 + rnd() * 120.0);
                let g = ctx.create_radial_gradient(x, y, 0, x, y, r);
                g.add_color_stop(0, &format!("rgba(70,60,44,{:.3})", 0.04 + 0.07 * rnd()));
                g.add_color_stop(1, "rgba(70,60,44,0)");
                ctx.set_fill_style(&g);
                ctx.fill_rect(0, 0, w, h);
            }
            let ge = ctx.create_linear_gradient(0, h * 0.75, 0, h);
            ge.add_color_stop(0, "rgba(60,50,38,0)");
            ge.add_color_stop(1, "rgba(60,50,38,0.22)");
            ctx.set_fill_style(&ge);
            ctx.fill_rect(0, 0, w, h);
            // scuffs: fine scratches through the print to the white plastic,
            // most on its lower half where things brushed past
            ctx.set_line_cap("round");
            for _ in 0..140 {
                let (x, y) = (rnd() * w, h * (1.0 - rnd().powf(1.6)));
                let a = (rnd() - 0.5) * 1.2 + if rnd() < 0.5 { 0.0 } else { PI / 2.0 };
                let l = 3.0 + rnd() * rnd() * 40.0;
                ctx.set_stroke_style(format!("rgba(232,228,214,{:.2})", 0.25 + 0.45 * rnd()));
                ctx.set_line_width(0.4 + rnd() * 0.9);
                ctx.begin_path();
                ctx.move_to(x, y);
                ctx.quadratic_curve_to(
                    x + a.cos() * l * 0.5 + (rnd() - 0.5) * 3.0,
                    y + a.sin() * l * 0.5,
                    x + a.cos() * l,
                    y + a.sin() * l,
                );
                ctx.stroke();
            }
            // a few scrapes where the ink came off in a patch
            for _ in 0..4 {
                let (x, y) = (rnd() * w, h * (0.3 + 0.7 * rnd()));
                for _ in 0..12 {
                    ctx.set_fill_style(format!("rgba(226,222,206,{:.2})", 0.3 + 0.4 * rnd()));
                    ctx.begin_path();
                    ctx.ellipse(
                        x + (rnd() - 0.5) * 16.0,
                        y + (rnd() - 0.5) * 8.0,
                        1.0 + rnd() * 4.0,
                        0.5 + rnd() * 1.5,
                        rnd() * 0.5,
                        0,
                        PI * 2.0,
                    );
                    ctx.fill();
                }
            }
            // rust run from each screw (15 mm in from each side)
            for (sx, sy) in [(0.015, 0.015), (0.195, 0.015), (0.015, 0.282), (0.195, 0.282)] {
                let (x, y) = (sx * ppm, sy * ppm);
                let len = 20.0 + rnd() * 60.0;
                let g = ctx.create_linear_gradient(x, y, x, y + len);
                g.add_color_stop(0, &format!("rgba(104,52,22,{:.2})", 0.35 + 0.3 * rnd()));
                g.add_color_stop(1, "rgba(104,52,22,0)");
                ctx.set_fill_style(&g);
                ctx.begin_path();
                ctx.move_to(x - 4.0, y);
                ctx.line_to(x + 4.0, y);
                ctx.line_to(x + 1.5 + (rnd() - 0.5) * 3.0, y + len);
                ctx.line_to(x - 1.5 + (rnd() - 0.5) * 3.0, y + len);
                ctx.close_path();
                ctx.fill();
                let stops = [(0.0, "rgba(90,44,18,0.5)".to_string()), (1.0, "rgba(90,44,18,0)".to_string())];
                let gr = ctx.create_radial_gradient(x, y, 0, x, y, 12.0);
                for (at, c) in &stops {
                    gr.add_color_stop(*at, c);
                }
                ctx.set_fill_style(&gr);
                ctx.fill_rect(x - 12.0, y - 12.0, 24.0, 24.0);
            }
            // dust along its top edge
            let gd = ctx.create_linear_gradient(0, 0, 0, 14);
            gd.add_color_stop(0, "rgba(150,144,132,0.5)");
            gd.add_color_stop(1, "rgba(150,144,132,0)");
            ctx.set_fill_style(&gd);
            ctx.fill_rect(0, 0, w, 14);
        })
    })
}

// R515 teddy paw: 6% chance per level. Dismembered plush paw + torn stuffing.
pub(crate) fn place_teddy_paws(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let fur_colors = [0x8a5030, 0x604020, 0x503020];
    let fur_mat = lambert!({ color: pick_hex(&fur_colors) });
    let stuffing_mat = lambert!({ color: 0xe8e0d0, transparent: true, opacity: 0.85 });
    let seam_mat = lambert!({ color: 0x2a1810 });
    let paw = group();
    // Paw ball (sphere).
    let ball = mesh(pg("TeddyPaws__paw", sphere_geo!(0.020, 8, 6)), &fur_mat);
    ball.scale().set(1, 1.4, 1);
    ball.position().y = 0.020;
    paw.add(&ball);
    // Torn edge: faces up into the ragged crater on the paw's top.
    let torn = mesh(circle_geo!(0.018, 8), &seam_mat);
    torn.rotation().x = -PI / 2.0;
    torn.scale().set_scalar(0.62);
    torn.position().y = 0.0445;
    paw.add(&torn);
    // Stuffing puffs (3 small clouds nearby).
    for i in 0..3 {
        let puff = mesh(sphere_geo!(0.008, 6, 5), &stuffing_mat);
        puff.scale().set(1.0 + rnd() * 0.5, 0.6, 1.0 + rnd() * 0.5);
        puff.position().set((rnd() - 0.5) * 0.08, 0.006, 0.04 + (rnd() - 0.5) * 0.04);
        paw.add(&puff);
    }
    scatter_on_floor(g, &paw, x, z, 2.4);
    paw.rotation().z = (rnd() - 0.5) * 0.4;
    g.world.add(&paw);
}

// R514 caution tape strip: 5% chance per level. Striped tape on wall.
pub(crate) fn place_caution_tapes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    let mat = lambert!({ map: caution_tape_texture(), side: DoubleSide });
    let tape = mesh(plane_geo!(2.2, 0.05), &mat);
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.005);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.005);
    let center_y = 1.1;
    if c.dx.abs() == 1 {
        tape.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        tape.position().set(wall_x, center_y, wall_z);
    } else {
        tape.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        tape.position().set(wall_x, center_y, wall_z);
    }
    // Small tilt for organic hang.
    tape.rotation().z = (rnd() - 0.5) * 0.08;
    g.world.add(&tape);
}

// R514 broken green bottle: 6% chance per level. Base + shards + neck fragment.
pub(crate) fn place_broken_bottles(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let glass_colors = [0x306040, 0x603820, 0x304060];
    let glass_mat = basic_mat!({ color: pick_hex(&glass_colors), transparent: true, opacity: 0.7 });
    let grp = group();
    // Base stub (bottom cylinder).
    let base = mesh(cyl_geo!(0.024, 0.028, 0.020, 10), &glass_mat);
    base.position().y = 0.010;
    grp.add(&base);
    // Jagged rim tabs on base.
    for i in 0..4 {
        let a = (i as f64 / 4.0) * PI * 2.0 + rnd() * 0.4;
        let tab = mesh(box_geo!(0.008, 0.010 + rnd() * 0.008, 0.003), &glass_mat);
        tab.position().set(a.cos() * 0.024, 0.025, a.sin() * 0.024);
        tab.rotation().y = -a;
        grp.add(&tab);
    }
    // Separated neck fragment nearby.
    let neck = mesh(cyl_geo!(0.008, 0.011, 0.035, 8), &glass_mat);
    neck.rotation().z = PI / 2.0;
    neck.position().set(0.08, 0.008, 0.05);
    grp.add(&neck);
    // scattered shards.
    for i in 0..6 {
        let s1 = 0.014 + rnd() * 0.016;
        let shard_geo = tri_geo([0.0, 0.0, 0.0, s1, 0.0, s1 * (rnd() - 0.5), s1 * 0.5, 0.0, s1 * (0.5 + rnd() * 0.5)]);
        let shard = mesh(shard_geo, &glass_mat);
        shard.position().set((rnd() - 0.5) * 0.20, 0.002, (rnd() - 0.5) * 0.20);
        shard.rotation().y = rnd() * PI * 2.0;
        grp.add(&shard);
    }
    scatter_on_floor(g, &grp, x, z, 2.0);
    g.world.add(&grp);
}

// R513 chalk arrow: 5% chance per level. A way-mark someone left on a wall at
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

// R513 shattered eyeglasses: 6% chance per level. Frame + lenses + broken temple.
pub(crate) fn place_shattered_eyeglasses(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let frame_mat = lambert!({ color: 0x181008 });
    let lens_mat = basic_mat!({ color: 0xc0d0e0, transparent: true, opacity: 0.4 });
    let glasses = group();
    // 2 lens frames (torus).
    for s in [-1.0, 1.0] {
        let rim = mesh(pg("ShatteredEyeglasses__rim", torus_geo!(0.020, 0.002, 4, 12)), &frame_mat);
        rim.rotation().x = PI / 2.0;
        rim.position().set(s * 0.024, 0.002, 0);
        glasses.add(&rim);
        // Lens disc.
        let lens = mesh(circle_geo!(0.018, 12), &lens_mat);
        lens.rotation().x = -PI / 2.0;
        lens.position().set(s * 0.024, 0.003, 0);
        glasses.add(&lens);
    }
    // Bridge between lenses.
    let bridge = mesh(pg("ShatteredEyeglasses__bridge", cyl_geo!(0.001, 0.001, 0.010, 3)), &frame_mat);
    bridge.rotation().z = PI / 2.0;
    bridge.position().set(0, 0.002, 0);
    glasses.add(&bridge);
    // 1 temple attached (other broken off).
    let temple_side = if rnd() < 0.5 { -1.0 } else { 1.0 };
    let temple = mesh(pg("ShatteredEyeglasses__temple", cyl_geo!(0.001, 0.001, 0.10, 3)), &frame_mat);
    temple.rotation().x = PI / 2.0;
    temple.rotation().z = 0.1;
    temple.position().set(temple_side * 0.044, 0.002, -0.05);
    glasses.add(&temple);
    // Broken temple laying separately.
    let broken = mesh(pg("ShatteredEyeglasses__broken", cyl_geo!(0.001, 0.001, 0.06, 3)), &frame_mat);
    broken.rotation().z = PI / 3.0;
    broken.position().set(-temple_side * 0.10, 0.002, 0.04);
    glasses.add(&broken);
    scatter_on_floor(g, &glasses, x, z, 2.4);
    g.world.add(&glasses);
}

// R512 wooden mouse trap: 7% chance per level. Wood base + spring bar + cheese.
pub(crate) fn place_mouse_traps(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    } // 7% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let wood_mat = lambert!({ color: 0xa08050 });
    let wire_mat = lambert!({ color: 0xc0c0c0 });
    let cheese_mat = lambert!({ color: 0xf0d040 });
    let trap = group();
    // Base plank.
    let base = mesh(pg("MouseTraps__base", box_geo!(0.06, 0.006, 0.038)), &wood_mat);
    base.position().y = 0.003;
    trap.add(&base);
    // Spring bar (U-shape wire).
    let spring_loop = mesh(pg("MouseTraps__snap", torus_geo!(0.016, 0.001, 4, 8)), &wire_mat);
    spring_loop.position().set(-0.015, 0.010, 0);
    spring_loop.rotation().x = PI / 2.0;
    trap.add(&spring_loop);
    // Bar wire across base.
    let bar = mesh(pg("MouseTraps__bar", cyl_geo!(0.001, 0.001, 0.036, 4)), &wire_mat);
    bar.rotation().x = PI / 2.0;
    bar.position().set(0.010, 0.007, 0);
    trap.add(&bar);
    // Trigger tab.
    let trigger = mesh(pg("MouseTraps__trigger", box_geo!(0.006, 0.002, 0.014)), &wire_mat);
    trigger.position().set(0.020, 0.008, 0);
    trap.add(&trigger);
    // Cheese bait cube.
    let cheese = mesh(pg("MouseTraps__cheese", box_geo!(0.006, 0.005, 0.005)), &cheese_mat);
    cheese.position().set(0.020, 0.009, 0);
    trap.add(&cheese);
    scatter_on_floor(g, &trap, x, z, 2.4);
    g.world.add(&trap);
}

// R512 wall crayon graffiti: 5% chance per level. Child-drawn scene on wall.
pub(crate) fn place_crayon_graffiti(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    let mat = basic_mat!({ map: crayon_graffiti_texture(), transparent: true, opacity: 0.85, depthWrite: false });
    let decal = mesh(plane_geo!(0.60, 0.50), &mat);
    mount_on_wall(g, &decal, &c, 0.008, 0.9, 0.6); // low-mounted (child-height)
    g.world.add(&decal);
}

// R511 family portrait: 5% chance per level. Framed photo w/ 3-4 silhouettes.
pub(crate) fn place_family_portraits(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    let frame_mat = lambert!({ color: 0x6a3820 });
    let photo_mat = lambert!({ map: family_portrait_texture(), side: DoubleSide });
    let portrait = group();
    let (w, h, t, fw) = (0.20, 0.16, 0.015, 0.014);
    for (bar_w, bar_h, bx, by) in frame_bars(w, h, fw) {
        let bar = mesh(pg("FamilyPortraits__bar", box_geo!(bar_w, bar_h, t)), &frame_mat);
        bar.position().set(bx, by, 0);
        portrait.add(&bar);
    }
    let photo = mesh(plane_geo!(w, h), &photo_mat);
    photo.position().set(0, 0, t / 2.0 + 0.001);
    portrait.add(&photo);
    mount_on_wall(g, &portrait, &c, 0.010, 1.60, 0.6);
    portrait.rotation().z = (rnd() - 0.5) * 0.05;
    g.world.add(&portrait);
}

// R511 cassette tape: 8% chance per level. Rectangle w/ 2 spool holes + label.
pub(crate) fn place_cassette_tapes(g: &mut Game, _dist: &[i32]) {
    g.pp.cassette_positions.clear(); // R524 reset per level
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let body_colors = [0x181410, 0x201810, 0x4a2818, 0x503060];
    let body_mat = lambert!({ color: pick_hex(&body_colors) });
    let label_colors = [0xf0ece0, 0xd8b040, 0xa02040, 0xd0d0d0];
    let label_mat = lambert!({ color: pick_hex(&label_colors) });
    let hole_mat = basic_mat!({ color: 0x080604 });
    let cass = group();
    // Body: rectangular block.
    let body = mesh(pg("CassetteTapes__body", box_geo!(0.10, 0.010, 0.06)), &body_mat);
    body.position().y = 0.005;
    cass.add(&body);
    // Label on top.
    let label = mesh(plane_geo!(0.08, 0.035), &label_mat);
    label.rotation().x = -PI / 2.0;
    label.position().set(0, 0.011, -0.010);
    cass.add(&label);
    // 2 spool holes.
    for s in [-1.0, 1.0] {
        let hole = mesh(circle_geo!(0.008, 8), &hole_mat);
        hole.rotation().x = -PI / 2.0;
        hole.position().set(s * 0.024, 0.011, 0.014);
        cass.add(&hole);
    }
    scatter_on_floor(g, &cass, x, z, 2.4);
    g.world.add(&cass);
    // R524 track for hiss audio.
    let p = cass.pos();
    g.pp.cassette_positions.push(v3(p.x, 0.01, p.z));
}

// R510 mop bucket: 6% chance per level. Plastic bucket with a mop in it.
pub(crate) fn place_mop_buckets(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let bucket_colors = [0x2050c0, 0x304050, 0xa02040];
    let bucket_mat = lambert!({ color: pick_hex(&bucket_colors) });
    let handle_mat = lambert!({ color: 0x606870 });
    let stick_mat = lambert!({ color: 0x8a5030 });
    let mop_mat = lambert!({ color: 0xa08a70 });
    let water_mat = basic_mat!({ color: 0x304050, transparent: true, opacity: 0.7 });
    let bucket = group();
    // Bucket body (open cylinder, wider at top).
    let body = mesh(pg("MopBuckets__body", cyl_geo!(0.075, 0.055, 0.15, 12, 1, true)), &bucket_mat);
    body.position().y = 0.075;
    bucket.add(&body);
    // Base disc.
    let base = mesh(circle_geo!(0.055, 12), &bucket_mat);
    base.rotation().x = -PI / 2.0;
    base.position().y = 0.001;
    bucket.add(&base);
    // Water inside (60% chance).
    if rnd() < 0.60 {
        let water = mesh(circle_geo!(0.070, 12), &water_mat);
        water.rotation().x = -PI / 2.0;
        water.scale().set(0.93, 0.93, 1); // inside the wall at this height
        water.position().y = 0.10;
        bucket.add(&water);
    }
    // Metal handle (torus arc).
    let handle = mesh(pg("MopBuckets__handle", torus_geo!(0.075, 0.003, 4, 12, PI)), &handle_mat);
    handle.position().y = 0.150;
    handle.rotation().x = PI / 2.0;
    bucket.add(&handle);
    // Mop leaning against bucket rim: foot in the mop head inside the bucket.
    let stick_len = 0.60;
    let stick = mesh(pg("MopBuckets__stick", cyl_geo!(0.008, 0.008, stick_len, 6)), &stick_mat);
    stick.rotation().z = -0.5;
    stick.position().set(0.012 + (0.5f64).sin() * stick_len / 2.0, 0.05 + (0.5f64).cos() * stick_len / 2.0, 0);
    bucket.add(&stick);
    // Mop head (dirty bundle on the stick's foot, sitting in the bucket).
    let mop_head = mesh(pg("MopBuckets__mopHead", sphere_geo!(0.045, 8, 6)), &mop_mat);
    mop_head.scale().set(0.9, 1.2, 0.9);
    mop_head.position().set(0.012, 0.058, 0);
    bucket.add(&mop_head);
    scatter_on_floor(g, &bucket, x, z, 1.6);
    g.world.add(&bucket);
}

// R510 dust bunnies: 10% chance per level. Where no air moves, what a floor
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

// The loose fibres round a dust bunny: lint every way on nothing, cut out
// by the alpha test (laid on its fuzz shell by its box faces); made once,
// shared by every bunny
game::lazy_static_tex!(
    dust_fringe_tex,
    canvas_tex(160, 160, |ctx, w, h| {
        ctx.clear_rect(0, 0, w, h);
        ctx.set_line_cap("round");
        for k in 0..520 {
            let (x, y) = (rnd() * w, rnd() * h);
            let a = rnd() * PI * 2.0;
            let l = 8.0 + rnd() * 30.0;
            let v = 120.0 + rnd() * 90.0;
            ctx.set_stroke_style(if k % 41 == 0 { "rgb(104,112,136)".to_string() } else { format!("rgb({v:.0},{v:.0},{:.0})", v - 3.0) });
            ctx.set_line_width(0.9 + rnd() * 1.1);
            ctx.begin_path();
            ctx.move_to(x, y);
            let bend = (rnd() - 0.5) * 1.6;
            ctx.quadratic_curve_to(x + (a + bend).cos() * l * 0.5, y + (a + bend).sin() * l * 0.5, x + a.cos() * l, y + a.sin() * l);
            ctx.stroke();
        }
    })
);

// The fibres a dust bunny is felted of, laid on its parts by their box faces:
// grey and white lint, dark hairs, a fleck of coloured wool, grit; made
// once, shared by every bunny
game::lazy_static_tex!(
    dust_fluff_tex,
    canvas_tex(192, 192, |ctx, w, h| {
        ctx.set_fill_style("rgb(128,128,127)");
        ctx.fill_rect(0, 0, w, h);
        // the felt: fibres every way, light and dark greys
        ctx.set_line_cap("round");
        for k in 0..900 {
            let (x, y) = (rnd() * w, rnd() * h);
            let a = rnd() * PI * 2.0;
            let l = 6.0 + rnd() * 26.0;
            let v = 110.0 + rnd() * 110.0;
            let tint = if k % 37 == 0 {
                // a fibre off a jumper or a carpet: faded blue, red, ochre
                ["rgba(96,108,140,0.6)", "rgba(140,90,82,0.6)", "rgba(150,128,86,0.6)"][(rnd() * 3.0) as usize % 3].to_string()
            } else {
                format!("rgba({v:.0},{v:.0},{:.0},{:.2})", v - 3.0, 0.25 + rnd() * 0.4)
            };
            ctx.set_stroke_style(&tint);
            ctx.set_line_width(0.6 + rnd() * 1.2);
            ctx.begin_path();
            ctx.move_to(x, y);
            let bend = (rnd() - 0.5) * 1.2;
            ctx.quadratic_curve_to(x + (a + bend).cos() * l * 0.5, y + (a + bend).sin() * l * 0.5, x + a.cos() * l, y + a.sin() * l);
            ctx.stroke();
        }
        // hairs, dark and long
        for _ in 0..24 {
            let (x, y) = (rnd() * w, rnd() * h);
            let a = rnd() * PI * 2.0;
            let l = 40.0 + rnd() * 80.0;
            ctx.set_stroke_style(format!("rgba(40,34,30,{:.2})", 0.35 + rnd() * 0.3));
            ctx.set_line_width(0.6);
            ctx.begin_path();
            ctx.move_to(x, y);
            ctx.quadratic_curve_to(x + (a + 0.6).cos() * l * 0.5, y + (a + 0.6).sin() * l * 0.5, x + a.cos() * l, y + a.sin() * l);
            ctx.stroke();
        }
        // grit and the dark of the felt's hollows
        for _ in 0..260 {
            let (x, y) = (rnd() * w, rnd() * h);
            let v = 50.0 + rnd() * 50.0;
            ctx.set_fill_style(format!("rgba({v:.0},{:.0},{:.0},{:.2})", v - 4.0, v - 10.0, 0.2 + rnd() * 0.4));
            ctx.fill_rect(x, y, 1.0 + rnd() * 1.5, 1.0 + rnd() * 1.5);
        }
    })
);

/// The dust under a clutch of dust bunnies on a plane 0.46 m square: a thin
/// grey film thickest round them (`spots`, m, from its middle: x, z and how
/// big), and the fibres and hairs come loose from them; on carpet the film
/// hardly shows, the fibres do
fn dust_film_tex(spots: Vec<(f64, f64, f64)>, hard: bool, vary: u32) -> Tex {
    let side = 0.46;
    mark_tex(256.0, 256.0, [0.5, 0.495, 0.485], vary, move |ctx, w, h| {
        let seed = (rnd() * 4.0e9) as u32;
        let s = side / w;
        let film_k = if hard { 0.3 } else { 0.1 };
        let mut img = ctx.create_image_data(w, h);
        let n = w as usize;
        let spots2 = spots.clone();
        par_rows(&mut img.data, n * 4, |j, out| {
            let y = (j as f64 + 0.5) * s - side / 2.0;
            for i in 0..n {
                let x = (i as f64 + 0.5) * s - side / 2.0;
                let mut near = 0.0f64;
                for &(bx, bz, r) in &spots2 {
                    near += (-((x - bx).hypot(y - bz) / (r * 2.5 + 0.03)).powi(2)).exp();
                }
                let cloud = fbm(seed, x * 18.0, y * 18.0, 4);
                let edge = sstep(0.0, 0.08, side / 2.0 - x.abs().max(y.abs()));
                let a = (near.min(1.0) * 0.8 + 0.25 * cloud) * sstep(0.25, 0.75, cloud + 0.3 * near.min(1.0)) * film_k * edge;
                let v = 0.48 + 0.06 * vnoise(seed ^ 0x9, x * 300.0, y * 300.0);
                let c = [v, v * 0.99, v * 0.97, a];
                for (k, q) in c.iter().enumerate() {
                    out[i * 4 + k] = (q.clamp(0.0, 1.0) * 255.0).round() as u8;
                }
            }
        });
        ctx.put_image_data(&img, 0, 0);
        // the loose fibres and hairs, near the bunnies mostly
        ctx.set_line_cap("round");
        let to_px = |m: f64| (m + side / 2.0) / s;
        for k in 0..70 {
            let (bx, bz, r) = spots[k % spots.len()];
            let a = rnd() * PI * 2.0;
            let d = r * (0.6 + 2.5 * rnd() * rnd());
            let (x, y) = (to_px(bx + a.cos() * d), to_px(bz + a.sin() * d));
            let hair = k % 9 == 0;
            let l = if hair { 30.0 + rnd() * 40.0 } else { 5.0 + rnd() * 14.0 };
            let b = rnd() * PI * 2.0;
            let v = 110.0 + rnd() * 70.0;
            ctx.set_stroke_style(if hair {
                format!("rgba(42,38,36,{:.2})", 0.35 + rnd() * 0.25)
            } else {
                format!("rgba({v:.0},{v:.0},{:.0},{:.2})", v - 3.0, 0.2 + rnd() * 0.3)
            });
            ctx.set_line_width(if hair { 0.5 } else { 0.5 + rnd() * 0.5 });
            ctx.begin_path();
            ctx.move_to(x, y);
            let bend = (rnd() - 0.5) * 1.6;
            ctx.quadratic_curve_to(x + (b + bend).cos() * l * 0.5, y + (b + bend).sin() * l * 0.5, x + b.cos() * l, y + b.sin() * l);
            ctx.stroke();
        }
    })
}

// R509 open coloring book: 6% chance per level. Cover + open spread + crayon.
pub(crate) fn place_coloring_books(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let cover_mat = lambert!({ color: 0xd82060 });
    let page_mat = lambert!({ map: coloring_texture(), side: DoubleSide });
    let book = group();
    // Cover under.
    let cover = mesh(pg("ColoringBooks__cover", box_geo!(0.22, 0.003, 0.14)), &cover_mat);
    cover.position().y = 0.002;
    book.add(&cover);
    // Pages.
    let pages = mesh(plane_geo!(0.20, 0.13), &page_mat);
    pages.rotation().x = -PI / 2.0;
    pages.position().y = 0.005;
    book.add(&pages);
    // Crayon nearby.
    let crayon_colors = [0xd82040, 0x2050d8, 0xd8b040];
    let crayon_mat = lambert!({ color: pick_hex(&crayon_colors) });
    let crayon = mesh(pg("ColoringBooks__crayon", cyl_geo!(0.004, 0.004, 0.05, 6)), &crayon_mat);
    crayon.rotation().z = PI / 2.0;
    crayon.position().set(0.08, 0.004, -0.05);
    crayon.rotation().y = 0.5;
    book.add(&crayon);
    scatter_on_floor(g, &book, x, z, 2.0);
    g.world.add(&book);
}

// R509 loose vinyl record: 8% chance per level. Black disc w/ coloured label.
pub(crate) fn place_vinyl_record_discs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let vinyl_mat = lambert!({ color: 0x101008 });
    let label_colors = [0xa02040, 0x2050d8, 0xd8b040, 0x30a030];
    let label_mat = lambert!({ color: pick_hex(&label_colors) });
    let disc = group();
    // Main vinyl.
    let body = mesh(pg("VinylRecordDiscs__disc", cyl_geo!(0.078, 0.078, 0.002, 20)), &vinyl_mat);
    body.position().y = 0.001;
    disc.add(&body);
    // Label.
    let label = mesh(circle_geo!(0.025, 12), &label_mat);
    label.rotation().x = -PI / 2.0;
    label.position().y = 0.0025;
    disc.add(&label);
    // Center hole (tiny dark spot).
    let hole = mesh(circle_geo!(0.003, 6), &vinyl_mat);
    hole.rotation().x = -PI / 2.0;
    hole.position().y = 0.0028;
    disc.add(&hole);
    scatter_on_floor(g, &disc, x, z, 2.4);
    g.world.add(&disc);
}

// R508 broken picture frame: 6% chance per level. Frame face-down + glass shards.
pub(crate) fn place_broken_picture_frames(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let frame_colors = [0x603820, 0x8a5030, 0x2a1810, 0xd8b040];
    let frame_mat = lambert!({ color: pick_hex(&frame_colors) });
    let glass_mat = basic_mat!({ color: 0xc0d0e0, transparent: true, opacity: 0.5 });
    let back_mat = lambert!({ color: 0xe8dcc0 });
    let grp = group();
    let (w, h, t, fw) = (0.22, 0.28, 0.015, 0.024);
    // Frame face-down (positive Y = back visible).
    let frame = group();
    for (bar_w, bar_h, bx, by) in frame_bars(w, h, fw) {
        let bar = mesh(pg("BrokenPictureFrames__bar", box_geo!(bar_w, bar_h, t)), &frame_mat);
        bar.position().set(bx, by, 0);
        frame.add(&bar);
    }
    // Paper backing.
    let back = mesh(plane_geo!(w, h), &back_mat);
    back.rotation().y = PI; // faces out of the back (-z)
    back.position().set(0, 0, -t / 2.0 - 0.001);
    frame.add(&back);
    // face-down as intended: front (+z) to the floor, backing on top
    frame.rotation().x = PI / 2.0;
    frame.position().y = 0.008;
    grp.add(&frame);
    // 5-8 scattered glass shards around frame.
    let n_shards = 5 + (rnd() * 4.0).floor() as i32;
    for i in 0..n_shards {
        let s1 = 0.020 + rnd() * 0.020;
        let shard_geo = tri_geo([0.0, 0.0, 0.0, s1, 0.0, s1 * (rnd() - 0.5) * 0.5, s1 * 0.5, 0.0, s1 * (0.5 + rnd() * 0.5)]);
        let shard = mesh(shard_geo, &glass_mat);
        shard.position().set((rnd() - 0.5) * 0.36, 0.002, (rnd() - 0.5) * 0.36);
        shard.rotation().y = rnd() * PI * 2.0;
        grp.add(&shard);
    }
    scatter_on_floor(g, &grp, x, z, 1.8);
    g.world.add(&grp);
}

// R507 opened suitcase: 5% chance per level. Base + lifted lid + clothes piles.
pub(crate) fn place_opened_suitcases(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let leather_colors = [0x603820, 0x502010, 0x8a5030, 0x201a10];
    let leather_mat = lambert!({ color: pick_hex(&leather_colors) });
    let brass_mat = lambert!({ color: 0xc89040 });
    let cloth_colors = [0xd82040, 0x2050d8, 0xd8b040, 0x30a030, 0xa020a0, 0xf0ece0, 0x2a2418];
    let suitcase = group();
    let (bw, bh, bd) = (0.22, 0.05, 0.14);
    // Base (bottom half).
    let base = mesh(pg("OpenedSuitcases__base", box_geo!(bw, bh, bd)), &leather_mat);
    base.position().y = bh / 2.0;
    suitcase.add(&base);
    // Lid (raised, tilted open), pivoting on its back bottom edge (the hinge).
    let lid = group();
    let lid_box = mesh(pg("OpenedSuitcases__lid", box_geo!(bw, bh, bd)), &leather_mat);
    lid_box.position().set(0, bh / 2.0, bd / 2.0);
    lid.add(&lid_box);
    lid.position().set(0, bh, -bd / 2.0);
    lid.rotation().x = -PI / 3.0;
    suitcase.add(&lid);
    // Brass corner reinforcements at 4 corners of base.
    for (cx, cz) in [(-1.0, -1.0), (-1.0, 1.0), (1.0, -1.0), (1.0, 1.0)] {
        let corner = mesh(pg("OpenedSuitcases__stud", cyl_geo!(0.008, 0.008, 0.008, 5)), &brass_mat);
        corner.position().set(cx * (bw / 2.0 - 0.004), 0.004, cz * (bd / 2.0 - 0.004));
        suitcase.add(&corner);
    }
    // 4-6 clothes piles inside base.
    let n_clothes = 4 + (rnd() * 3.0).floor() as i32;
    for i in 0..n_clothes {
        let cloth_mat = lambert!({ color: pick_hex(&cloth_colors) });
        // Wrinkled cloth.
        let cloth =
            mesh(pg("OpenedSuitcases__cloth", box_geo!(0.05 + rnd() * 0.04, 0.008 + rnd() * 0.006, 0.03 + rnd() * 0.03)), &cloth_mat);
        cloth.position().set((rnd() - 0.5) * (bw - 0.05), bh + 0.005 + i as f64 * 0.003, (rnd() - 0.5) * (bd - 0.03));
        cloth.rotation().y = rnd() * PI * 2.0;
        cloth.rotation().z = (rnd() - 0.5) * 0.2;
        suitcase.add(&cloth);
    }
    scatter_on_floor(g, &suitcase, x, z, 1.8);
    g.world.add(&suitcase);
}

// R506 rusted tin can: 8% chance per level. Small rusted can, sometimes tipped.
pub(crate) fn place_rusted_tin_cans(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let can_colors = [0x6a4028, 0x804830, 0x8a3020, 0x503020];
    let can_mat = lambert!({ color: pick_hex(&can_colors) });
    let label_mat = lambert!({ color: 0xa08a70 });
    let can = group();
    // Can body.
    let body = mesh(pg("RustedTinCans__body", cyl_geo!(0.028, 0.028, 0.075, 12)), &can_mat);
    body.position().y = 0.038;
    can.add(&body);
    // Faded paper label band.
    let label = mesh(pg("RustedTinCans__label", cyl_geo!(0.029, 0.029, 0.035, 12, 1, true)), &label_mat);
    label.position().y = 0.038;
    can.add(&label);
    // Rim discs top + bottom.
    for yy in [0.003, 0.072] {
        let rim = mesh(cyl_geo!(0.028, 0.028, 0.002, 12), &can_mat);
        rim.position().y = yy;
        can.add(&rim);
    }
    // 40% tipped on side.
    if rnd() < 0.40 {
        can.rotation().z = PI / 2.0;
        can.position().y = 0.028;
    }
    can.position().x = g.wx(x) + (rnd() - 0.5) * 2.4;
    can.position().z = g.wz(z) + (rnd() - 0.5) * 2.4;
    can.rotation().y = rnd() * PI * 2.0;
    g.world.add(&can);
}

// R505 wall taxidermy deer head: 4% chance per level. Deer bust on a plaque.
pub(crate) fn place_deer_head_mounts(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    let plaque_mat = lambert!({ color: 0x603820 });
    let deer_mat = lambert!({ color: 0x8a6540 });
    let antler_mat = lambert!({ color: 0xe8dcc0 });
    let eye_mat = basic_mat!({ color: 0x100804 });
    let mount = group();
    // Wood plaque (shield shape approximation).
    let plaque = mesh(pg("DeerHeadMounts__plaque", box_geo!(0.28, 0.32, 0.020)), &plaque_mat);
    plaque.position().set(0, 0, -0.014);
    mount.add(&plaque);
    // Deer head (elongated sphere).
    let head = mesh(pg("DeerHeadMounts__head", sphere_geo!(0.070, 12, 10)), &deer_mat);
    head.scale().set(0.85, 1.2, 0.90);
    head.position().set(0, 0, 0.040);
    mount.add(&head);
    // Snout.
    let snout = mesh(pg("DeerHeadMounts__snout", sphere_geo!(0.040, 8, 6)), &deer_mat);
    snout.scale().set(0.7, 0.7, 1);
    snout.position().set(0, -0.055, 0.075);
    mount.add(&snout);
    // Nose.
    let nose = mesh(sphere_geo!(0.008, 6, 5), &eye_mat);
    nose.position().set(0, -0.075, 0.105);
    mount.add(&nose);
    // 2 eyes.
    for s in [-1.0, 1.0] {
        let eye = mesh(sphere_geo!(0.006, 5, 4), &eye_mat);
        eye.position().set(s * 0.024, 0.010, 0.088);
        mount.add(&eye);
    }
    // 2 ears (small triangles): tips lean outwards, base on the head's crown.
    for s in [-1.0, 1.0] {
        let ear = mesh(pg("DeerHeadMounts__ear", cone_geo!(0.014, 0.030, 4)), &deer_mat);
        ear.position().set(s * 0.050, 0.065, 0.020);
        ear.rotation().z = -s * 0.5;
        mount.add(&ear);
    }
    // 2 antlers - each with branches (Y-shape approximation).
    for s in [-1.0, 1.0] {
        // Main stem.
        let stem = mesh(pg("DeerHeadMounts__beam", cyl_geo!(0.005, 0.005, 0.10, 4)), &antler_mat);
        stem.position().set(s * 0.025, 0.100, 0.020);
        stem.rotation().z = s * -0.3;
        mount.add(&stem);
        // Branches.
        for b in 0..3 {
            let b = b as f64;
            let branch = mesh(pg("DeerHeadMounts__tine", cyl_geo!(0.003, 0.003, 0.05, 4)), &antler_mat);
            branch.position().set(s * (0.040 + b * 0.010), 0.100 + b * 0.020 - 0.010, 0.020);
            branch.rotation().z = s * -0.9;
            mount.add(&branch);
        }
    }
    mount_on_wall(g, &mount, &c, 0.005, 2.00, 0.6);
    g.world.add(&mount);
}
