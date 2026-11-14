//! `place_mold_patches`
use game::prelude::*;
use placers_kit::noise::fbm;
use placers_kit::noise::hash;
use placers_kit::noise::specks;
use placers_kit::noise::sstep;
use placers_kit::paint::decal_tex;
use placers_kit::paint::lay;
use placers_kit::paint::mark_mat;
use placers_kit::place::mount_slid;
use placers_kit::place::seed_of;
use placers_kit::place::wall_candidates;
use placers_kit::place::wall_xz;

// Mould: black mould grown on a damp wall (`assets/src/blender/props/
// docs/research/props-remodelling/wall_marks_and_wear.md`). Colonies of Cladosporium / Stachybotrys: each a cluster
// of dark specks (1-3 mm) grown out from where it began, packed into a sooty,
// velvety film at its old middle, thinning to peppered dots at its edge,
// overlapping into an irregular patch; satellites round it; under it the
// faint yellow-brown stain of the damp that fed it, with its tide line.
// Painted on a transparent plane the old decal's size, laid on the wall.
pub(crate) fn place_mold_patches(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = choice(&candidates);
    let (pw, ph) = (0.60, 0.80);
    // the plane: the old decal's, its face 1 mm off the wall (it was 8 mm)
    let geo = plane_geo!(pw, ph);
    geo.translate(0.0, 0.0, -0.007);
    let decal = mesh(geo, &lambert!({ color: 0x808080 }));
    let (wall_x, wall_z) = wall_xz(g, c, 0.008);
    // its height (the old draw)
    let center_y = 1.2 + rnd() * 0.6;
    mount_slid(&decal, c, wall_x, wall_z, center_y, 0.5);
    // what it looks like: from where it is (no draws)
    let p = decal.pos();
    let damp = matches!(g.weather_key(), "wet" | "derelict" | "haunted");
    decal.set_mat(mould_mat(seed_of(&[p.x, p.y, p.z, c.dx as f64, c.dz as f64]), pw, ph, damp));
    g.world.add(&decal);
}

/// A small generator for a mark's own layout, seeded from where it is
struct Rng(pub u32);

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

fn mix(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

/// The mould across a plane `pw` x `ph` m (y down from its top), from `seed`;
/// heavier where the wall stays `damp`.
fn mould_mat(seed: u32, pw: f64, ph: f64, damp: bool) -> Mat {
    let mut r = Rng(seed | 1);
    // the colonies: (x, y, radius, maturity, squash); the oldest near the
    // middle, younger ones and satellites out from it, and a trail or two of
    // them down the wall where the damp ran
    let (cx, cy) = (pw / 2.0 + (r.f() - 0.5) * 0.10, ph * 0.42 + (r.f() - 0.5) * 0.12);
    let spread = if damp { 1.15 } else { 1.0 };
    let mut cols: Vec<(f64, f64, f64, f64, f64)> = vec![];
    let keep = |x: f64, y: f64, rad: f64, sq: f64| {
        // kept clear of the plane's edges, so nothing is cut by them
        let m = 0.03 + rad * 2.3;
        let (mx, my) = (m.min(pw / 2.0), (m / sq.min(1.0)).min(ph / 2.0));
        (x.clamp(mx, pw - mx), y.clamp(my, ph - my))
    };
    let ring = |r: &mut Rng, cols: &mut Vec<(f64, f64, f64, f64, f64)>, n: usize, d0: f64, d1: f64, r0: f64, r1: f64, a0: f64, a1: f64| {
        for _ in 0..n {
            let ang = r.f() * std::f64::consts::TAU;
            let d = (d0 + (d1 - d0) * r.f()) * spread;
            let rad = r0 + (r1 - r0) * r.f();
            let sq = 0.7 + 0.5 * r.f();
            let (x, y) = keep(cx + ang.cos() * d * 1.1, cy + ang.sin() * d * 0.9, rad, sq);
            cols.push((x, y, rad, a0 + (a1 - a0) * r.f(), sq));
        }
    };
    let n = 2 + (r.f() * 2.0) as usize;
    ring(&mut r, &mut cols, n, 0.03, 0.15, 0.06, 0.1, 0.85, 1.0);
    let n = 5 + (r.f() * 5.0) as usize;
    ring(&mut r, &mut cols, n, 0.1, 0.26, 0.025, 0.055, 0.6, 0.95);
    let n = 8 + (r.f() * 6.0) as usize;
    ring(&mut r, &mut cols, n, 0.15, 0.3, 0.008, 0.022, 0.3, 0.75);
    // trails down the wall: thin colonies strung below the patch
    for _ in 0..1 + (r.f() * 2.2) as usize {
        let x0 = cx + (r.f() - 0.5) * 0.18;
        let len = 0.12 + 0.18 * r.f();
        let mut y = cy + 0.06;
        while y < cy + 0.06 + len {
            let rad = 0.004 + 0.007 * r.f() * (1.0 - (y - cy) / 0.5).max(0.3);
            let (x, yy) = keep(x0 + (r.f() - 0.5) * 0.02, y, rad, 0.5);
            cols.push((x, yy, rad, 0.25 + 0.3 * r.f(), 0.5));
            y += rad * (1.5 + 2.5 * r.f());
        }
    }
    let s = r.u();
    // (sRGB) the old black at a colony's middle, its olive-grey edge, the
    // pale fuzz here and there, the damp's stain and its tide line
    const CORE: [f64; 3] = [0.075, 0.08, 0.066];
    const EDGE: [f64; 3] = [0.30, 0.31, 0.24];
    const FUZZ: [f64; 3] = [0.6, 0.6, 0.53];
    const STAIN: [f64; 3] = [0.5, 0.42, 0.26];
    const TIDE: [f64; 3] = [0.4, 0.31, 0.17];
    let (tw, th) = (384, 512);
    let tex = decal_tex(tw, th, pw, ph, CORE, move |x, y, aa| {
        // how grown it is here (0-1), and how far the damp reached
        let warp = fbm(s, x * 22.0, y * 22.0, 2) - 0.5;
        let (mut dn, mut dh) = (0.0f64, 0.0f64);
        for (k, &(mx, my, mr, ma, sq)) in cols.iter().enumerate() {
            let q = (x - mx).hypot((y - my) * sq) / mr * (1.0 + 0.7 * warp + 0.25 * (hash(s, k as i32, 7) - 0.5));
            if q < 3.5 {
                dn = 1.0 - (1.0 - dn) * (1.0 - ma * (-q * q * 0.9).exp());
            }
            dh = dh.max(ma * (-(q / 2.3).powi(2)).exp());
        }
        let mut px = [CORE[0], CORE[1], CORE[2], 0.0];
        // the damp's stain under it, and the line where it stopped
        let dh = dh * (0.8 + 0.4 * fbm(s ^ 0x55, x * 9.0, y * 9.0, 2));
        lay(&mut px, STAIN, 0.15 * sstep(0.05, 0.35, dh));
        lay(&mut px, TIDE, 0.16 * (-((dh - 0.1) / 0.02).powi(2)).exp());
        // the velvety haze among the specks, and the sooty film at the oldest middles
        let mottle = fbm(s ^ 0x77, x * 70.0, y * 70.0, 3);
        lay(&mut px, mix(EDGE, CORE, 0.55), 0.36 * sstep(0.2, 0.9, dn) * (0.35 + 0.65 * mottle));
        lay(&mut px, CORE, 0.45 * sstep(0.72, 1.0, dn) * sstep(0.35, 0.75, mottle));
        // the specks: a dot in a cell as likely as the colony is dense there
        let layers: [(f64, f64, f64, f64, u32); 4] = [
            // cell, likelihood, radius, more radius, salt
            (0.0015, 0.9 * sstep(0.03, 0.45, dn), 0.0002, 0.00025, 0x1001),
            (0.003, 0.7 * sstep(0.12, 0.6, dn), 0.0004, 0.0005, 0x1002),
            (0.006, 0.5 * sstep(0.3, 0.8, dn) * (0.5 + mottle), 0.0008, 0.001, 0x1003),
            (0.008, 0.35 * sstep(0.004, 0.03, dn) * (1.0 - sstep(0.1, 0.3, dn)), 0.0003, 0.0004, 0x1004),
        ];
        for &(cell, like, r0, r1, salt) in &layers {
            if like <= 0.001 {
                continue;
            }
            let (dot, hk) = specks(s ^ salt, x, y, cell, like, r0, r1, aa);
            if dot > 0.0 {
                let shade = mix(EDGE, CORE, (dn * 1.3 + (hk - 0.5) * 0.6).clamp(0.0, 1.0));
                lay(&mut px, shade, dot * 0.92);
            }
        }
        // pale fuzz here and there where it is growing
        let (fz, _) = specks(s ^ 0x1005, x, y, 0.008, 0.10 * sstep(0.2, 0.5, dn) * (1.0 - sstep(0.7, 0.9, dn)), 0.0012, 0.0004, 0.0012);
        lay(&mut px, FUZZ, fz * 0.3);
        // nothing at the plane's edges
        px[3] *= sstep(0.0, 0.025, x.min(pw - x).min(y).min(ph - y));
        px
    });
    mark_mat(&tex)
}
