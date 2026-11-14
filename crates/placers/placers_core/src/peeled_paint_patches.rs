//! `place_peeled_paint_patches`
use game::prelude::*;
use placers_kit::noise::Rng32;
use placers_kit::noise::fbm;
use placers_kit::noise::hash;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::decal_tex;
use placers_kit::paint::lay;
use placers_kit::paint::mark_mat;
use placers_kit::place::mount;
use placers_kit::place::seed_of;
use placers_kit::place::wall_candidates;
use placers_kit::place::wall_key_at;
use placers_kit::place::wall_xz;

// Peeled paint (`docs/research/props-remodelling/wall_marks_and_wear.md`): where damp
// came through a painted wall the paint let go: cracked into flakes along
// a network of fine cracks, the flakes come away in patches showing the
// older coat beneath, and under that the plaster (or the board's paper), a
// white bloom of salts on it; the edges of what is left a real step (the
// film's broken edge catching the light, its shadow below), lifted here
// and there, a few flakes curled right off the wall (modelled). On bare
// masonry it is what is left of an old coat: an island of paint flaking
// away from the middle out. Painted on a plane the old decal's size.
pub(crate) fn place_peeled_paint_patches(g: &mut Game, _dist: &[i32]) {
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
    let mut r = Rng32(seed_of(&[p.x, p.y, p.z, c.dx as f64, c.dz as f64]));
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
    let decal = mesh(plane_geo!(pw, ph), mark_mat(&tex));
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

/// sRGB (0-1) as a hex colour
fn hex_of(c: [f64; 3]) -> u32 {
    c.iter().fold(0u32, |h, v| (h << 8) | (v.clamp(0.0, 1.0) * 255.0).round() as u32)
}

/// The paint's flakes on a patch `pw` x `ph` (x right, y up, from its
/// middle): cells of a cracked film (jittered Voronoi, its edges wavering),
/// each gone or still there by how far the damp got (`env`), and under the
/// gone ones the older coat, itself gone in places.
struct Flakes {
    pub seed: u32,
    pub painted: bool,
    pub pw: f64,
    pub ph: f64,
    /// the patch's middle and its half extents
    pub mid: (f64, f64),
    pub reach: (f64, f64),
}

impl Flakes {
    const CELL: f64 = 0.017;
    const SUB: f64 = 0.014;

    pub fn new(seed: u32, painted: bool, pw: f64, ph: f64) -> Self {
        let mut r = Rng32(seed | 1);
        let mid = ((r.f() - 0.5) * 0.06, (r.f() - 0.5) * 0.08);
        let reach = (0.18 + 0.03 * r.f(), 0.23 + 0.04 * r.f());
        Flakes { seed: r.u(), painted, pw, ph, mid, reach }
    }

    /// How far the damp got, 0-1 (none by the plane's edges)
    pub fn env(&self, x: f64, y: f64) -> f64 {
        let (dx, dy) = ((x - self.mid.0) / self.reach.0, (y - self.mid.1) / self.reach.1);
        let q = dx.dhypot(dy) * (1.0 + 0.5 * (fbm(self.seed ^ 0x61, x * 7.0, y * 7.0, 3) - 0.5));
        sstep(1.0, 0.25, q) * sstep(0.0, 0.03, (self.pw / 2.0 - x.abs()).min(self.ph / 2.0 - y.abs()))
    }

    /// Whether a cell (its point at `c`, its hash `h`) has lost its paint
    pub fn cell_gone(&self, c: (f64, f64), h: f64) -> bool {
        let e = self.env(c.0, c.1);
        if self.painted { h < 0.9 * sstep(0.06, 0.5, e) } else { h >= 0.97 * sstep(0.0, 0.2, e) }
    }

    /// The nearest cell point to `q` on a grid of `cell` (each cell's point
    /// in its middle 70%, so the nearest is among the 3 x 3 round it), and
    /// the nearest edge to a cell in the other state (`state`): (its state,
    /// the distance to that edge, the edge's normal toward the other cell,
    /// the distance to any edge, this cell, the other cell)
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
            let c = ((f64::from(i) + 0.15 + 0.7 * hash(s, i, j)) * cell, (f64::from(j) + 0.15 + 0.7 * hash(s ^ 0x5, i, j)) * cell);
            let d = (q.0 - c.0).dhypot(q.1 - c.1);
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
            let l = vx.dhypot(vy).max(1e-9);
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
            x + 0.004 * (fbm(self.seed ^ 0x71, x * 55.0, y * 55.0, 2) - 0.5) + 0.0012 * (vn(self.seed ^ 0x72, x * 260.0, y * 260.0) - 0.5),
            y + 0.004 * (fbm(self.seed ^ 0x73, x * 55.0, y * 55.0, 2) - 0.5) + 0.0012 * (vn(self.seed ^ 0x74, x * 260.0, y * 260.0) - 0.5),
        );
        let s = self.seed;
        let st = |c: (f64, f64), i: i32, j: i32| self.cell_gone(c, hash(s ^ 0x11, i, j));
        let (gone, d, n0, crack, ija, ijb) = self.voronoi(0x1, Self::CELL, q, &st);
        // n0 points from this cell to the other: make it from what is there to what is gone
        let n = if gone { (-n0.0, -n0.1) } else { n0 };
        // the cell still there beside the edge (this one, or the one across it) is lifted at it
        let kept = if gone { ijb } else { ija };
        let lifted = hash(s ^ 0x13, kept.0, kept.1) < 0.45;
        let env = self.env(x, y);
        // the older coat, gone in its own smaller flakes where the damp was worst
        let (deep, dd, dn) = if gone && self.painted {
            let sd = |c: (f64, f64), i: i32, j: i32| hash(s ^ 0x21, i, j) < 0.4 * sstep(0.5, 1.0, self.env(c.0, c.1)) + 0.04;
            let (dp, dd, dn0, _, _, _) = self.voronoi(0x2, Self::SUB, q, &sd);
            (dp, dd, if dp { (-dn0.0, -dn0.1) } else { dn0 })
        } else {
            (false, f64::INFINITY, (0.0, 1.0))
        };
        Here { gone, d, n, lifted, crack, deep, dd, dn, env }
    }

    /// Where flakes have curled off: 3-5 points on the edge of a hole, each
    /// (x, y, its turn on the wall, which flake, its size)
    pub fn curls(&self, r: &mut Rng32) -> Vec<(f64, f64, f64, usize, f64)> {
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
            if out.iter().any(|o| (o.0 - x).dhypot(o.1 - y) < 0.05) {
                continue;
            }
            // the flake's free side (its +y) into the hole
            let ang = (-n.0).datan2(n.1);
            out.push((x, y, ang, (r.f() * 3.0) as usize % 3, 0.85 + 0.3 * r.f()));
        }
        out
    }
}

/// What the film is at a point
struct Here {
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

/// The patch painted: holes in the top coat (on a painted wall: the wall
/// itself is the top coat, clear here) or the island of paint (on
/// masonry), their edges, the coats and plaster under them, the curled
/// flakes' shadows.
fn peeled_paint_tex(f: &Flakes, coat: [f64; 3], base: [f64; 3], curls: &[(f64, f64, f64, usize, f64)]) -> Tex {
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
            let (nx, ny) = (-ang.dsin(), ang.dcos());
            (x + nx * l * 0.35, y + ny * l * 0.35 - 0.004, l * 0.55)
        })
        .collect();
    let (pw, ph, painted, s) = (f.pw, f.ph, f.painted, f.seed);

    decal_tex(400, 520, pw, ph, coat, move |cx, cy, aa| {
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
                    lay(&mut px, LIT, 0.4 * h.dn.1.max(0.0) * sstep(0.0007, 0.0, h.dd));
                } else if h.deep && h.dd < 0.002 {
                    lay(&mut px, SHADE, 0.35 * (-h.dn.1).max(0.0) * sstep(0.0012, 0.0, h.dd));
                }
                // dust and grime settled in it
                lay(&mut px, [0.3, 0.27, 0.22], 0.18 * fbm(s ^ 0x85, x * 30.0, y * 30.0, 3));
                // salts the damp brought out, worst in the middle
                lay(&mut px, SALT, 0.35 * sstep(0.5, 0.9, h.env) * sstep(0.55, 0.8, fbm(s ^ 0x82, x * 40.0, y * 40.0, 3)));
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
            lay(&mut px, SHADE, (0.55 * below + 0.12) * (-h.d / w).dexp() * sstep(-aa, 0.0, h.d));
        } else {
            // the film's broken edge catching the light where it faces up
            let up = h.n.1.max(0.0);
            lay(&mut px, LIT, (0.5 * up + 0.15) * sstep(0.0009, 0.0002, h.d));
            // a lifted edge: tipped toward the light or away from it
            if h.lifted {
                let k = sstep(0.004, 0.0, h.d);
                if h.n.1 > 0.0 {
                    lay(&mut px, LIT, 0.3 * h.n.1 * k);
                } else {
                    lay(&mut px, SHADE, 0.25 * -h.n.1 * k);
                }
            }
            // the cracks the film broke along before it let go
            let ck = sstep(0.1, 0.45, h.env) * sstep(0.25, 0.6, vn(s ^ 0x84, x * 90.0, y * 90.0));
            lay(&mut px, SHADE, 0.45 * ck * (0.5 - (h.crack - 0.00025) / aa).clamp(0.0, 1.0));
        }
        // the curled flakes' shadows
        for &(sx, sy, sr) in &shadows {
            let q = (x - sx).dhypot((y - sy) * 1.4) / sr;
            if q < 1.5 {
                lay(&mut px, SHADE, 0.32 * (-q * q * 2.0).dexp());
            }
        }
        px
    })
}
