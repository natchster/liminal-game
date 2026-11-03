//! Batch `r11b`'s props in this crate, made real (`docs/PROPS.md` P2): its
//! placers and what only they use.
use super::*;

// R552 peeled paint (`assets/src/blender/props/r11b.notes.md`): where damp
// came through a painted wall the paint let go: cracked into flakes along
// a network of fine cracks, the flakes come away in patches showing the
// older coat beneath, and under that the plaster (or the board's paper), a
// white bloom of salts on it; the edges of what is left a real step (the
// film's broken edge catching the light, its shadow below), lifted here
// and there, a few flakes curled right off the wall (modelled). On bare
// masonry it is what is left of an old coat: an island of paint flaking
// away from the middle out. Painted on a plane the old decal's size.
pub(super) fn place_peeled_paint_patches(g: &mut Game, _dist: &[i32]) {
    use r11b_kit::*;
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = choice(&candidates);
    let (pw, ph) = (0.50, 0.65);
    let patch = group();
    let (wall_x, wall_z) = wall_xz(g, c, 0.008);
    mount(&patch, c, wall_x, wall_z, 1.4, Some(0.5));
    // what it is, from where it is (no draws)
    let p = patch.pos();
    let mut r = Rng(seed_of(&[p.x, p.y, p.z, c.dx as f64, c.dz as f64]));
    let key = wall_key_at(g, c.x, c.z);
    let painted = key.starts_with("wp:") || key.starts_with("drywall") || key.starts_with("paint");
    let flakes = Flakes::new(r.u(), painted, pw, ph);
    // the coats: the older one under the top coat (or the island's paint on
    // masonry), what is under that, and the back of a flake
    const COATS: [[f64; 3]; 4] = [[0.53, 0.58, 0.5], [0.71, 0.66, 0.53], [0.55, 0.59, 0.61], [0.69, 0.68, 0.64]];
    let coat = COATS[(r.f() * 4.0) as usize % 4];
    let base = if key.starts_with("drywall") { [0.62, 0.58, 0.5] } else { [0.72, 0.7, 0.64] };
    // a few flakes curled off the wall, at the edges of the holes
    let curls = flakes.curls(&mut r);
    let tex = peeled_paint_tex(&flakes, coat, base, &curls);
    let decal = mesh(plane_geo!(pw, ph), &decal_mat(&tex));
    decal.position().z = -0.007;
    patch.add(&decal);
    // the flakes: the top coat's face (the wall's own surface, or the
    // island's paint) and the film's back
    let top = if painted { g.arch_mat(&key).duplicate() } else { lambert!({ color: hex_of(coat) }) };
    let under = lambert!({ color: 0xcfcabd });
    for &(x, y, ang, k, sc) in &curls {
        for (part, mat) in [("Top", &top), ("Under", &under)] {
            let name = format!("Real__peeledPaintFlake{}{part}", ["A", "B", "C"][k]);
            let geo = game::assets::props::prop_part_as_modelled(&name).unwrap_or_else(|| box_geo!(0.03, 0.03, 0.01));
            let m = mesh(geo, mat);
            m.position().set(x, y, -0.008);
            m.rotation().z = ang;
            m.scale().set(sc, sc, sc);
            patch.add(&m);
        }
    }
    // a mark on the wall: the placement pass leaves it where the settle put it
    patch.user().floats = true;
    g.world.add(&patch);
}

// R545 wall pipe leak (`assets/src/blender/props/r11b.notes.md`): a short
// run of 2 in steel pipe out of the wall, along it and back in (batch r06's
// malleable elbows, a coupling, the collars at the wall), painted once and
// rusting; one elbow's thread by the wall weeps: a crust of lime and rust
// under it, a drop hanging, and down the wall from its collar the wet run,
// rust-brown at its edges, and the stain where it spread and dried.
pub(super) fn place_wall_pipe_leaks(g: &mut Game, _dist: &[i32]) {
    use r11b_kit::*;
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = choice(&candidates);
    let leak = group();
    // the old pipe's axis 2.6 cm off the wall: the group's origin stays there
    let (wall_x, wall_z) = wall_xz(g, c, 0.026);
    mount(&leak, c, wall_x, wall_z, 1.90, Some(0.3));
    let p = leak.pos();
    let mut r = Rng(seed_of(&[p.x, p.y, p.z, c.dx as f64, c.dz as f64]));
    // the frame: the wall's face at z = WALL; the run's axis 85 mm off it, as
    // high as keeps the collars' tops where the old pipe's top was; its
    // elbows' collars at the old pipe's ends
    const WALL: f64 = -0.026;
    const EA: f64 = 0.057; // a 2 in elbow, centre to end
    const IN: f64 = 0.017; // a thread into its fitting
    let (zp, ya, xe) = (WALL + 0.085, 0.025927 - 0.060, 0.300 - 0.0575);
    let paint = (r.f() * 3.0) as usize % 3;
    let pipe_mat = lambert!({ map: &pipe_paint_tex(paint, false), color: 0xffffff });
    let fit_mat = lambert!({ map: &pipe_paint_tex(paint, true), color: 0xffffff });
    let plate = phong!({ color: 0x77766f, specular: 0x262626, shininess: 26 });
    let part = |name: &str, w: f64| game::assets::props::prop_part_as_modelled(name).unwrap_or_else(|| box_geo!(w, w, w));
    // the run, between the elbows, through a coupling
    let xc = (r.f() - 0.5) * 0.12;
    leak.add(&pipe_length(&pipe_mat, v3(-xe + EA - IN, ya, zp), v3(xc - 0.015, ya, zp)));
    leak.add(&pipe_length(&pipe_mat, v3(xc + 0.015, ya, zp), v3(xe - EA + IN, ya, zp)));
    let cp = mesh(part("Real__pipeCoupling", 0.064), &fit_mat);
    cp.position().set(xc, ya, zp);
    leak.add(&cp);
    for sx in [-1.0, 1.0] {
        let x = sx * xe;
        // the elbow (its legs +x and +y as modelled): one leg along the run,
        // the other back into the wall
        let el = group();
        el.position().set(x, ya, zp);
        el.rotation().y = if sx < 0.0 { 0.0 } else { PI };
        let m = mesh(part("Real__pipeElbow", 0.08), &fit_mat);
        // its +y leg to -z (into the wall); the right one is turned round
        // after (its +x leg to -x), so its +y leg goes to +z first
        m.rotation().x = sx * PI / 2.0;
        el.add(&m);
        leak.add(&el);
        leak.add(&pipe_length(&pipe_mat, v3(x, ya, zp - EA + IN), v3(x, ya, WALL)));
        let col = mesh(part("Real__pipeCollar", 0.115), &plate);
        col.position().set(x, ya, WALL);
        leak.add(&col);
    }
    // the leaking one, at its thread by the wall
    let side = if r.f() < 0.5 { -1.0 } else { 1.0 };
    let lx = side * xe;
    let crust = mesh(part("Real__pipeLeakCrust", 0.03), &lambert!({ color: 0x9c8466 }));
    crust.position().set(lx, ya - 0.0385, zp - EA + 0.006);
    crust.rotation().y = r.f() * 0.6 - 0.3;
    leak.add(&crust);
    let drop = mesh(
        part("Real__pipeLeakDrop", 0.006),
        &phong!({ color: 0x4a5256, specular: 0xd8dcdc, shininess: 120, transparent: true, opacity: 0.55 }),
    );
    drop.position().set(lx - 0.003, ya - 0.0385 - 0.0128, zp - EA + 0.007);
    leak.add(&drop);
    // down the wall from its collar: the wet run and its stain, to where
    // the old stain's foot was
    let (top, bot) = (ya + 0.03, -0.26);
    let (w, hgt) = (0.10, top - bot);
    let run = mesh(plane_geo!(w, hgt), &decal_mat(&pipe_leak_tex(r.u(), w, hgt, top - (ya - 0.0575))));
    run.position().set(lx, (top + bot) / 2.0, WALL + 0.001);
    leak.add(&run);
    g.world.add(&leak);
}

/// Batch r11b's painting kit (the marks on the walls): hashed noise, a seeded
/// generator for a mark's layout (no level draws), and a per-pixel painter
/// filled on every core (`par_rows`).
mod r11b_kit {
    use game::prelude::*;

    /// A small generator for a mark's own layout, seeded from where it is
    pub struct Rng(pub u32);
    impl Rng {
        pub fn u(&mut self) -> u32 {
            let mut x = self.0;
            x ^= x << 13;
            x ^= x >> 17;
            x ^= x << 5;
            self.0 = x;
            x
        }
        /// in [0, 1)
        pub fn f(&mut self) -> f64 {
            f64::from(self.u()) / 4_294_967_296.0
        }
    }

    /// A seed from values already fixed (a position, a side): no draws
    pub fn seed_of(v: &[f64]) -> u32 {
        let mut x: u64 = 0xcbf2_9ce4_8422_2325;
        for a in v {
            for b in ((a * 1.0e4).round() as i64).to_le_bytes() {
                x ^= u64::from(b);
                x = x.wrapping_mul(0x0100_0000_01b3);
            }
        }
        ((x ^ (x >> 32)) as u32) | 1
    }

    /// A hash of (seed, i, j) in [0, 1)
    pub fn h01(seed: u32, i: i32, j: i32) -> f64 {
        let mut h = seed ^ (i as u32).wrapping_mul(0x27d4_eb2d) ^ (j as u32).wrapping_mul(0x1656_67b1);
        h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
        h = (h ^ (h >> 13)).wrapping_mul(0xc2b2_ae35);
        h ^= h >> 16;
        f64::from(h) / 4_294_967_296.0
    }

    /// Smooth value noise in [0, 1]
    pub fn vnoise(seed: u32, x: f64, y: f64) -> f64 {
        let (xf, yf) = (x.floor(), y.floor());
        let (i, j) = (xf as i32, yf as i32);
        let (u, v) = (x - xf, y - yf);
        let (u, v) = (u * u * (3.0 - 2.0 * u), v * v * (3.0 - 2.0 * v));
        let (a, b) = (h01(seed, i, j), h01(seed, i + 1, j));
        let (c, d) = (h01(seed, i, j + 1), h01(seed, i + 1, j + 1));
        a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v
    }

    /// Fractal value noise of `oct` octaves, in [0, 1]
    pub fn fbm(seed: u32, x: f64, y: f64, oct: u32) -> f64 {
        let (mut s, mut amp, mut f, mut n) = (0.0, 0.5, 1.0, 0.0);
        for k in 0..oct {
            s += amp * vnoise(seed.wrapping_add(k * 7919), x * f, y * f);
            n += amp;
            amp *= 0.5;
            f *= 2.03;
        }
        s / n
    }

    /// 0 below `e0`, 1 above `e1`, smooth between
    pub fn smooth(e0: f64, e1: f64, x: f64) -> f64 {
        let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }

    pub fn mix(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
        [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
    }

    /// `c` at alpha `a` over straight colour and alpha `dst`
    pub fn lay(dst: &mut [f64; 4], c: [f64; 3], a: f64) {
        let a = a.clamp(0.0, 1.0);
        if a <= 0.0 {
            return;
        }
        let out = a + dst[3] * (1.0 - a);
        if out > 1e-6 {
            for k in 0..3 {
                dst[k] = (c[k] * a + dst[k] * dst[3] * (1.0 - a)) / out;
            }
        }
        dst[3] = out;
    }

    /// Specks: a dot in each `cell` (m) with likelihood `like`, its radius
    /// `r0` + up to `r1`, anti-aliased over `aa`: (coverage, the dot's hash)
    #[allow(clippy::too_many_arguments)]
    pub fn specks(seed: u32, x: f64, y: f64, cell: f64, like: f64, r0: f64, r1: f64, aa: f64) -> (f64, f64) {
        let (gi, gj) = ((x / cell).floor() as i32, (y / cell).floor() as i32);
        let (mut best, mut hk) = (0.0f64, 0.0);
        for k in 0..9 {
            let (ci, cj) = (gi + k % 3 - 1, gj + k / 3 - 1);
            let h = h01(seed, ci, cj);
            if h >= like {
                continue;
            }
            let (px, py) = ((f64::from(ci) + h01(seed ^ 0x77, ci, cj)) * cell, (f64::from(cj) + h01(seed ^ 0x78, ci, cj)) * cell);
            let r = r0 + r1 * h01(seed ^ 0x79, ci, cj);
            let cov = (0.5 - ((x - px).hypot(y - py) - r) / aa).clamp(0.0, 1.0);
            if cov > best {
                best = cov;
                hk = h01(seed ^ 0x7a, ci, cj);
            }
        }
        (best, hk)
    }

    /// A transparent mark `tw` x `th` px across a plane `pw` x `ph` m: each
    /// pixel `px(x, y, aa)` (m from the plane's top left, y down; `aa` a
    /// pixel's size) in straight sRGB and alpha. Where it is clear its colour
    /// is `bleed`, so a mark's edge does not filter toward black.
    pub fn decal_tex(tw: usize, th: usize, pw: f64, ph: f64, bleed: [f64; 3], px: impl Fn(f64, f64, f64) -> [f64; 4] + Sync) -> Tex {
        let tex = canvas_tex_opt(tw as f64, th as f64, false, move |ctx, cw, ch| {
            let (sx, sy) = (pw / cw, ph / ch);
            let aa = sx.max(sy) * 1.1;
            let mut img = ctx.create_image_data(cw, ch);
            par_rows(&mut img.data, tw * 4, |j, out| {
                let y = (j as f64 + 0.5) * sy;
                for i in 0..tw {
                    let x = (i as f64 + 0.5) * sx;
                    let p = px(x, y, aa);
                    for (c, v) in p.iter().enumerate() {
                        out[i * 4 + c] = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
                    }
                }
            });
            ctx.put_image_data(&img, 0, 0);
        });
        // what is clear (it came back black through the canvas) takes the
        // bleed colour, and what is nearly clear goes toward it
        {
            let img = tex.image();
            let mut im = img.borrow_mut();
            for p in im.data.chunks_exact_mut(4) {
                let t = smooth(0.0, 0.06, f64::from(p[3]) / 255.0);
                for (c, b) in p.iter_mut().zip(bleed) {
                    *c = (b * 255.0 + (f64::from(*c) - b * 255.0) * t).round().clamp(0.0, 255.0) as u8;
                }
            }
            im.version += 1;
        }
        tex
    }

    /// The decal material: lit, see-through, drawn over the wall
    pub fn decal_mat(tex: &Tex) -> Mat {
        lambert!({ map: tex, transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 })
    }

    /// The surface of the wall seen from open tile (`x`, `z`): its arch
    /// material's key (`wp:<variant>` a papered wall), as the walls are drawn
    pub fn wall_key_at(g: &Game, x: i32, z: i32) -> String {
        if g.layout.is_none() || !g.in_grid(x, z) {
            return "wp:0".into();
        }
        g.wall_mat_key(g.t_idx(x, z))
    }

    /// An opaque, repeating surface `tw` x `th` px over `pw` x `ph` m: each
    /// pixel `px(x, y)` (m from its top left) in sRGB
    pub fn surface_tex(tw: usize, th: usize, pw: f64, ph: f64, px: impl Fn(f64, f64) -> [f64; 4] + Sync) -> Tex {
        canvas_tex_opt(tw as f64, th as f64, true, move |ctx, cw, ch| {
            let (sx, sy) = (pw / cw, ph / ch);
            let mut img = ctx.create_image_data(cw, ch);
            par_rows(&mut img.data, tw * 4, |j, out| {
                let y = (j as f64 + 0.5) * sy;
                for i in 0..tw {
                    let p = px((i as f64 + 0.5) * sx, y);
                    for (c, v) in p.iter().enumerate() {
                        out[i * 4 + c] = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
                    }
                }
            });
            ctx.put_image_data(&img, 0, 0);
        })
    }

    /// sRGB (0-1) as a hex colour
    pub fn hex_of(c: [f64; 3]) -> u32 {
        c.iter().fold(0u32, |h, v| (h << 8) | (v.clamp(0.0, 1.0) * 255.0).round() as u32)
    }

    /// The paint's flakes on a patch `pw` x `ph` (x right, y up, from its
    /// middle): cells of a cracked film (jittered Voronoi, its edges wavering),
    /// each gone or still there by how far the damp got (`env`), and under the
    /// gone ones the older coat, itself gone in places.
    pub struct Flakes {
        pub seed: u32,
        pub painted: bool,
        pub pw: f64,
        pub ph: f64,
        /// the patch's middle and its half extents
        pub mid: (f64, f64),
        pub reach: (f64, f64),
    }

    /// What the film is at a point
    pub struct Here {
        pub gone: bool,
        /// how far (m) to the edge between gone and still there
        pub d: f64,
        /// that edge's normal, from what is there into what is gone (y up)
        pub n: (f64, f64),
        /// the cell still there beside it is lifted at its edge
        pub lifted: bool,
        /// how far to the nearest crack between two cells
        pub crack: f64,
        /// the older coat gone here too, and how far to its edge, its normal
        pub deep: bool,
        pub dd: f64,
        pub dn: (f64, f64),
        /// how far the damp got (0-1)
        pub env: f64,
    }

    impl Flakes {
        const CELL: f64 = 0.017;
        const SUB: f64 = 0.014;

        pub fn new(seed: u32, painted: bool, pw: f64, ph: f64) -> Self {
            let mut r = Rng(seed | 1);
            let mid = ((r.f() - 0.5) * 0.06, (r.f() - 0.5) * 0.08);
            let reach = (0.18 + 0.03 * r.f(), 0.23 + 0.04 * r.f());
            Flakes { seed: r.u(), painted, pw, ph, mid, reach }
        }

        /// How far the damp got, 0-1 (none by the plane's edges)
        pub fn env(&self, x: f64, y: f64) -> f64 {
            let (dx, dy) = ((x - self.mid.0) / self.reach.0, (y - self.mid.1) / self.reach.1);
            let q = dx.hypot(dy) * (1.0 + 0.5 * (fbm(self.seed ^ 0x61, x * 7.0, y * 7.0, 3) - 0.5));
            smooth(1.0, 0.25, q) * smooth(0.0, 0.03, (self.pw / 2.0 - x.abs()).min(self.ph / 2.0 - y.abs()))
        }

        /// Whether a cell (its point at `c`, its hash `h`) has lost its paint
        pub fn cell_gone(&self, c: (f64, f64), h: f64) -> bool {
            let e = self.env(c.0, c.1);
            if self.painted {
                h < 0.9 * smooth(0.06, 0.5, e)
            } else {
                h >= 0.97 * smooth(0.0, 0.2, e)
            }
        }

        /// The nearest cell point to `q` on a grid of `cell` (each cell's point
        /// in its middle 70%, so the nearest is among the 3 x 3 round it), and
        /// the nearest edge to a cell in the other state (`state`): (its state,
        /// the distance to that edge, the edge's normal toward the other cell,
        /// the distance to any edge, this cell, the other cell)
        #[allow(clippy::type_complexity)]
        pub fn voronoi(
            &self,
            salt: u32,
            cell: f64,
            q: (f64, f64),
            state: &dyn Fn((f64, f64), i32, i32) -> bool,
        ) -> (bool, f64, (f64, f64), f64, (i32, i32), (i32, i32)) {
            let s = self.seed ^ salt;
            let (gi, gj) = ((q.0 / cell).floor() as i32, (q.1 / cell).floor() as i32);
            let mut pts = [((0.0, 0.0), (0, 0)); 9];
            let mut best = (f64::INFINITY, 0usize);
            for (k, pt) in pts.iter_mut().enumerate() {
                let (i, j) = (gi + k as i32 % 3 - 1, gj + k as i32 / 3 - 1);
                let c = ((f64::from(i) + 0.15 + 0.7 * h01(s, i, j)) * cell, (f64::from(j) + 0.15 + 0.7 * h01(s ^ 0x5, i, j)) * cell);
                let d = (q.0 - c.0).hypot(q.1 - c.1);
                *pt = (c, (i, j));
                if d < best.0 {
                    best = (d, k);
                }
            }
            let (a, ija) = pts[best.1];
            let ga = state(a, ija.0, ija.1);
            // the distance to the bisector with each other point
            let (mut d_edge, mut n, mut d_crack, mut ijb) = (f64::INFINITY, (0.0, 1.0), f64::INFINITY, ija);
            for (k, &(b, ij)) in pts.iter().enumerate() {
                if k == best.1 {
                    continue;
                }
                let (vx, vy) = (b.0 - a.0, b.1 - a.1);
                let l = vx.hypot(vy).max(1e-9);
                let d = ((a.0 + b.0) / 2.0 - q.0) * vx / l + ((a.1 + b.1) / 2.0 - q.1) * vy / l;
                d_crack = d_crack.min(d);
                if d < d_edge && state(b, ij.0, ij.1) != ga {
                    d_edge = d;
                    n = (vx / l, vy / l);
                    ijb = ij;
                }
            }
            (ga, d_edge, n, d_crack, ija, ijb)
        }

        pub fn at(&self, x: f64, y: f64) -> Here {
            // the cracks waver
            let q = (
                x + 0.004 * (fbm(self.seed ^ 0x71, x * 55.0, y * 55.0, 2) - 0.5)
                    + 0.0012 * (vnoise(self.seed ^ 0x72, x * 260.0, y * 260.0) - 0.5),
                y + 0.004 * (fbm(self.seed ^ 0x73, x * 55.0, y * 55.0, 2) - 0.5)
                    + 0.0012 * (vnoise(self.seed ^ 0x74, x * 260.0, y * 260.0) - 0.5),
            );
            let s = self.seed;
            let st = |c: (f64, f64), i: i32, j: i32| self.cell_gone(c, h01(s ^ 0x11, i, j));
            let (gone, d, n0, crack, ija, ijb) = self.voronoi(0x1, Self::CELL, q, &st);
            // n0 points from this cell to the other: make it from what is there to what is gone
            let n = if gone { (-n0.0, -n0.1) } else { n0 };
            // the cell still there beside the edge (this one, or the one across it) is lifted at it
            let kept = if gone { ijb } else { ija };
            let lifted = h01(s ^ 0x13, kept.0, kept.1) < 0.45;
            let env = self.env(x, y);
            // the older coat, gone in its own smaller flakes where the damp was worst
            let (deep, dd, dn) = if gone && self.painted {
                let sd = |c: (f64, f64), i: i32, j: i32| h01(s ^ 0x21, i, j) < 0.4 * smooth(0.5, 1.0, self.env(c.0, c.1)) + 0.04;
                let (dp, dd, dn0, _, _, _) = self.voronoi(0x2, Self::SUB, q, &sd);
                (dp, dd, if dp { (-dn0.0, -dn0.1) } else { dn0 })
            } else {
                (false, f64::INFINITY, (0.0, 1.0))
            };
            Here { gone, d, n, lifted, crack, deep, dd, dn, env }
        }

        /// Where flakes have curled off: 3-5 points on the edge of a hole, each
        /// (x, y, its turn on the wall, which flake, its size)
        pub fn curls(&self, r: &mut Rng) -> Vec<(f64, f64, f64, usize, f64)> {
            let mut found: Vec<(f64, f64, (f64, f64))> = vec![];
            let (nx, ny) = (36, 46);
            for j in 0..ny {
                for i in 0..nx {
                    let (x, y) =
                        ((i as f64 + 0.5) / nx as f64 * self.pw - self.pw / 2.0, (j as f64 + 0.5) / ny as f64 * self.ph - self.ph / 2.0);
                    if x.abs() > self.pw / 2.0 - 0.05 || y.abs() > self.ph / 2.0 - 0.05 {
                        continue;
                    }
                    let h = self.at(x, y);
                    // at a hole's edge, on the paint's side, a hole beyond it wide enough to have held the flake
                    if h.gone || h.d > 0.004 {
                        continue;
                    }
                    let (bx, by) = (x + h.n.0 * (h.d + 0.022), y + h.n.1 * (h.d + 0.022));
                    if !self.at(bx, by).gone || !self.at(x + h.n.0 * (h.d + 0.01), y + h.n.1 * (h.d + 0.01)).gone {
                        continue;
                    }
                    found.push((x + h.n.0 * h.d, y + h.n.1 * h.d, h.n));
                }
            }
            let mut out: Vec<(f64, f64, f64, usize, f64)> = vec![];
            let want = 3 + (r.f() * 3.0) as usize;
            let mut tries = 0;
            while out.len() < want && !found.is_empty() && tries < 60 {
                tries += 1;
                let (x, y, n) = found[(r.f() * found.len() as f64) as usize % found.len()];
                if out.iter().any(|o| (o.0 - x).hypot(o.1 - y) < 0.05) {
                    continue;
                }
                // the flake's free side (its +y) into the hole
                let ang = (-n.0).atan2(n.1);
                out.push((x, y, ang, (r.f() * 3.0) as usize % 3, 0.85 + 0.3 * r.f()));
            }
            out
        }
    }

    /// The patch painted: holes in the top coat (on a painted wall: the wall
    /// itself is the top coat, clear here) or the island of paint (on
    /// masonry), their edges, the coats and plaster under them, the curled
    /// flakes' shadows.
    pub fn peeled_paint_tex(f: &Flakes, coat: [f64; 3], base: [f64; 3], curls: &[(f64, f64, f64, usize, f64)]) -> Tex {
        const SHADE: [f64; 3] = [0.04, 0.035, 0.03];
        const LIT: [f64; 3] = [0.93, 0.92, 0.88];
        const SALT: [f64; 3] = [0.9, 0.9, 0.86];
        // the flakes' sizes (A, B, C as modelled): free length
        const LEN: [f64; 3] = [0.024, 0.016, 0.030];
        let shadows: Vec<(f64, f64, f64)> = curls
            .iter()
            .map(|&(x, y, ang, k, sc)| {
                let l = LEN[k] * sc;
                // the curl stands over the hole beside its foot; its shadow falls below it
                let (nx, ny) = (-ang.sin(), ang.cos());
                (x + nx * l * 0.35, y + ny * l * 0.35 - 0.004, l * 0.55)
            })
            .collect();
        let (pw, ph, painted, s) = (f.pw, f.ph, f.painted, f.seed);
        let tex = decal_tex(400, 520, pw, ph, coat, move |cx, cy, aa| {
            let (x, y) = (cx - pw / 2.0, ph / 2.0 - cy);
            let h = f.at(x, y);
            let mut px = [coat[0], coat[1], coat[2], 0.0];
            let grain = fbm(s ^ 0x81, x * 180.0, y * 180.0, 2);
            if h.gone == painted {
                // what shows in a hole (on a painted wall), or the island (on masonry)
                if painted {
                    let c = if h.deep {
                        let m = 0.9 + 0.14 * grain;
                        [base[0] * m, base[1] * m, base[2] * m]
                    } else {
                        let m = 0.94 + 0.08 * grain;
                        [coat[0] * m, coat[1] * m, coat[2] * m]
                    };
                    px = [c[0], c[1], c[2], (0.5 + h.d / aa).clamp(0.0, 1.0)];
                    // the older coat's own broken edge, and its shadow
                    if !h.deep && h.dd < 0.0015 {
                        lay(&mut px, LIT, 0.4 * h.dn.1.max(0.0) * smooth(0.0007, 0.0, h.dd));
                    } else if h.deep && h.dd < 0.002 {
                        lay(&mut px, SHADE, 0.35 * (-h.dn.1).max(0.0) * smooth(0.0012, 0.0, h.dd));
                    }
                    // dust and grime settled in it
                    lay(&mut px, [0.3, 0.27, 0.22], 0.18 * fbm(s ^ 0x85, x * 30.0, y * 30.0, 3));
                    // salts the damp brought out, worst in the middle
                    lay(&mut px, SALT, 0.35 * smooth(0.5, 0.9, h.env) * smooth(0.55, 0.8, fbm(s ^ 0x82, x * 40.0, y * 40.0, 3)));
                } else {
                    let m = 0.93 + 0.1 * grain;
                    px = [coat[0] * m, coat[1] * m, coat[2] * m, (0.5 + h.d / aa).clamp(0.0, 1.0)];
                    // grime on the old paint
                    lay(&mut px, [0.35, 0.32, 0.27], 0.2 * fbm(s ^ 0x83, x * 25.0, y * 25.0, 3));
                }
            }
            if h.gone {
                // in a hole, just past the edge: the shadow of the film's step
                // where the edge is above it (and wider under a lifted edge)
                let below = (-h.n.1).max(0.0);
                let w = if h.lifted { 0.0035 } else { 0.0012 };
                lay(&mut px, SHADE, (0.55 * below + 0.12) * (-h.d / w).exp() * smooth(-aa, 0.0, h.d));
            } else {
                // the film's broken edge catching the light where it faces up
                let up = h.n.1.max(0.0);
                lay(&mut px, LIT, (0.5 * up + 0.15) * smooth(0.0009, 0.0002, h.d));
                // a lifted edge: tipped toward the light or away from it
                if h.lifted {
                    let k = smooth(0.004, 0.0, h.d);
                    if h.n.1 > 0.0 {
                        lay(&mut px, LIT, 0.3 * h.n.1 * k);
                    } else {
                        lay(&mut px, SHADE, 0.25 * -h.n.1 * k);
                    }
                }
                // the cracks the film broke along before it let go
                let ck = smooth(0.1, 0.45, h.env) * smooth(0.25, 0.6, vnoise(s ^ 0x84, x * 90.0, y * 90.0));
                lay(&mut px, SHADE, 0.45 * ck * (0.5 - (h.crack - 0.00025) / aa).clamp(0.0, 1.0));
            }
            // the curled flakes' shadows
            for &(sx, sy, sr) in &shadows {
                let q = (x - sx).hypot((y - sy) * 1.4) / sr;
                if q < 1.5 {
                    lay(&mut px, SHADE, 0.32 * (-q * q * 2.0).exp());
                }
            }
            px
        });
        tex
    }

    /// A length of 2 in pipe (60.3 mm) from `a` to `b` along one axis, its
    /// paint wrapped round it at its real scale.
    pub fn pipe_length(mat: &Mat, a: Vec3, b: Vec3) -> Obj {
        let d = b - a;
        let len = d.length().max(0.001);
        let geo = cyl_geo!(0.03015, 0.03015, len, 16, 1, true);
        let uv: Vec<f32> = geo.r().uv.clone();
        // the texture is 0.4 m along
        let uv: Vec<f32> = uv.iter().enumerate().map(|(i, &v)| if i % 2 == 1 { v * (len / 0.4) as f32 } else { v }).collect();
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

    /// A pipe's old paint (`paint`: grey, green, cream) flaked off its steel
    /// in patches, rust in them and blooming round them, a run of rust along
    /// its underside, dust on its top; a fitting's (`fitting`) rustier.
    /// Round it 0.19 m, along it 0.4 m (one per kind, shared).
    pub fn pipe_paint_tex(paint: usize, fitting: bool) -> Tex {
        thread_local! { static T: std::cell::RefCell<Vec<Option<Tex>>> = std::cell::RefCell::new(vec![None; 6]); }
        let i = paint % 3 + if fitting { 3 } else { 0 };
        T.with(|t| {
            if let Some(x) = &t.borrow()[i] {
                return x.clone();
            }
            const PAINTS: [[f64; 3]; 3] = [[0.37, 0.38, 0.37], [0.34, 0.38, 0.31], [0.6, 0.57, 0.48]];
            let pc = PAINTS[paint % 3];
            let rusty = if fitting { 0.56 } else { 0.62 };
            let tex = surface_tex(96, 256, 0.19, 0.4, move |x, y| {
                let seed = 0x9e3 + paint as u32 * 31 + u32::from(fitting) * 7;
                // round the pipe: x 0-0.19 (a run's underside at 0.1425, its
                // top at 0.0475), along it: y
                let under = (-((x - 0.1425) / 0.03).powi(2)).exp();
                let top = (-((x - 0.0475) / 0.03).powi(2)).exp();
                let fl = fbm(seed, x * 70.0, y * 70.0, 4);
                let bare = smooth(rusty, rusty + 0.03, fl + 0.12 * under);
                let bloom = smooth(rusty - 0.12, rusty, fl + 0.12 * under) * (1.0 - bare);
                let rust = mix([0.24, 0.14, 0.08], [0.38, 0.21, 0.1], fbm(seed ^ 0x3, x * 300.0, y * 300.0, 2));
                let mut c = mix(pc, rust, bare);
                c = mix(c, [0.36, 0.22, 0.12], 0.4 * bloom);
                c = mix(c, [0.32, 0.19, 0.1], 0.35 * under);
                // grime
                c = mix(c, [0.2, 0.18, 0.15], 0.25 * fbm(seed ^ 0x7, x * 25.0, y * 25.0, 3));
                c = mix(c, [0.52, 0.5, 0.45], 0.25 * top);
                let m = 0.92 + 0.12 * fbm(seed ^ 0x5, x * 140.0, y * 30.0, 2);
                [c[0] * m, c[1] * m, c[2] * m, 1.0]
            });
            t.borrow_mut()[i] = Some(tex.clone());
            tex
        })
    }

    /// The run down the wall under a leaking collar, across a plane `pw` x `ph`
    /// (y down from its top; the collar's foot `from` down): wet and dark under
    /// the collar, rust at its edges, narrowing, then where it spread and
    /// dried, a stain with its tide lines.
    pub fn pipe_leak_tex(seed: u32, pw: f64, ph: f64, from: f64) -> Tex {
        let mut r = Rng(seed | 1);
        let s = r.u();
        const WET: [f64; 3] = [0.22, 0.16, 0.09];
        const STAIN: [f64; 3] = [0.5, 0.37, 0.2];
        const RUST: [f64; 3] = [0.45, 0.25, 0.1];
        const TIDE: [f64; 3] = [0.3, 0.18, 0.08];
        let (tw, th) = (96usize, 288usize);
        let sx = pw / tw as f64;
        let mid = pw / 2.0 + (r.f() - 0.5) * 0.008;
        // how far down it got below the collar's foot (m), in tongues: (x,
        // half width, depth), the deepest first
        let reach = ph - from - 0.02 - 0.03 * r.f();
        let lobes: Vec<(f64, f64, f64)> = (0..2 + (r.f() * 2.0) as usize)
            .map(|k| {
                if k == 0 {
                    (mid + (r.f() - 0.5) * 0.008, 0.024 + 0.01 * r.f(), reach)
                } else {
                    (mid + (r.f() - 0.5) * 0.05, 0.008 + 0.01 * r.f(), reach * (0.35 + 0.45 * r.f()))
                }
            })
            .collect();
        // each wetting's reach at each column, and its edge's slope; the
        // last (outermost) first, the earlier ones inside it
        let rims: Vec<f64> = vec![1.0, 0.78 + 0.08 * r.f(), 0.55 + 0.1 * r.f()];
        let reach_at = |x: f64, k: usize| -> f64 {
            let mut l = 0.004f64;
            for &(c, half, depth) in &lobes {
                let u = (x - c) / half;
                if u.abs() < 1.0 {
                    let tongue = depth * rims[k] * (1.0 - u.abs().powf(2.2)).powf(0.55);
                    let kk = 0.012;
                    let hh = (0.5 + 0.5 * (tongue - l) / kk).clamp(0.0, 1.0);
                    l = l + (tongue - l) * hh + kk * hh * (1.0 - hh);
                }
            }
            let wav =
                (fbm(s ^ (0x40 + k as u32), x * 40.0, 0.5, 3) - 0.5) * 0.03 + (vnoise(s ^ 0x41, x * 160.0, k as f64 * 3.0) - 0.5) * 0.004;
            (l + wav * (l / 0.03).min(1.0)).max(0.0)
        };
        let cols: Vec<Vec<(f64, f64)>> = (0..rims.len())
            .map(|k| {
                (0..tw)
                    .map(|i| {
                        let x = (i as f64 + 0.5) * sx;
                        let (l0, l1, l) = (reach_at(x - sx, k), reach_at(x + sx, k), reach_at(x, k));
                        (l, (1.0 + ((l1 - l0) / (2.0 * sx)).powi(2)).sqrt())
                    })
                    .collect()
            })
            .collect();
        decal_tex(tw, th, pw, ph, STAIN, move |x, y, aa| {
            let mut px = [STAIN[0], STAIN[1], STAIN[2], 0.0];
            let i = ((x / sx) as usize).min(tw - 1);
            // the damp round the hole behind the collar, seen past its rim
            let hole = (x - pw / 2.0).hypot(y - (from - 0.0575)) / 0.06;
            lay(&mut px, WET, 0.22 * smooth(1.25, 0.92, hole) * smooth(0.01, 0.03, y) * (0.6 + 0.4 * fbm(s ^ 0x1, x * 80.0, y * 80.0, 2)));
            let dy = y - from;
            // the wetted patch hanging from the collar's foot, and its rims:
            // sharp outside, what the water carried left along them
            let fine = (vnoise(s ^ 0x51, x * 260.0, y * 260.0) - 0.5) * 0.0015;
            let (l, slope) = cols[0][i];
            let top = (-dy - 0.006).max(0.0);
            let d0 = (dy - l) / slope + fine + top * 4.0 * smooth(0.012, 0.02, (x - mid).abs());
            let inside = (0.5 - d0 / aa).clamp(0.0, 1.0);
            let pale = 0.55 + 0.45 * (d0 / 0.012).exp();
            lay(&mut px, STAIN, inside * 0.45 * pale * (0.65 + 0.35 * fbm(s ^ 0x52, x * 50.0, y * 50.0, 2)));
            let tide = |d: f64| (0.5 - d / aa).clamp(0.0, 1.0) * (d.min(0.0) / 0.0018).exp();
            let along = |k: u32| 0.5 + 0.5 * fbm(s ^ (0x60 + k), x * 30.0, y * 12.0, 2);
            let mut lines = tide(d0) * along(0);
            for k in 1..rims.len() {
                let (lk, sk) = cols[k][i];
                lines = 1.0 - (1.0 - lines) * (1.0 - tide((dy - lk) / sk + fine) * 0.7 * along(k as u32) * inside);
            }
            lay(&mut px, TIDE, lines * 0.7);
            // still wet down its middle under the leak, rust at its edges
            let tt = (dy / reach).clamp(0.0, 1.0);
            let rw = 0.0045 * (1.0 - 0.5 * tt) * (0.6 + 0.8 * fbm(s ^ 0x6, 1.5, y * 20.0, 2));
            let xr = mid + (fbm(s ^ 0x7, 0.5, y * 11.0, 3) - 0.5) * 0.012 * tt;
            let dr = ((x - xr).abs() - rw).max(dy - l * 0.9).max(-dy - 0.004);
            lay(&mut px, WET, (0.5 - dr / aa).clamp(0.0, 1.0) * (0.4 - 0.2 * tt));
            lay(&mut px, RUST, 0.4 * smooth(-0.002, 0.0, dr) * smooth(0.0012, 0.0, dr));
            // nothing at the plane's edges
            px[3] *= smooth(0.0, 0.006, x.min(pw - x).min(y).min(ph - y));
            px
        })
    }
}
