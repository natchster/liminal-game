//! `place_rust_stain_streaks`
use game::prelude::*;
use placers_kit::noise::fbm;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::decal_tex;
use placers_kit::paint::lay;
use placers_kit::paint::mark_mat;
use placers_kit::place::mount_on_wall;
use placers_kit::place::seed_of;
use placers_kit::place::wall_candidates;

// Rust stain streak (`docs/research/props-remodelling/wall_marks_and_wear.md`): a
// steel fixing left in the wall (a stud anchor and its nut, or the corner
// brace a shelf stood on) rusting, and what the damp carried down from it: dark round
// the fixing where it began, then orange-brown runs, several side by side,
// paler and thinner as they go, each ending where it dried, a fainter wash
// between them. Painted on a transparent plane the old decal's size.
pub(crate) fn place_rust_stain_streaks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    let (pw, ph) = (0.24, 0.60);
    let grp = group();
    mount_on_wall(g, &grp, &c, 0.005, 1.5, 0.5);
    // what it is, from where it is (no draws)
    let p = grp.pos();
    let mut r = Rng(seed_of(&[p.x, p.y, p.z, c.dx as f64, c.dz as f64]));
    let bracket = r.f() < 0.4;
    // the fixing: a hand's breadth under the top of the old decal; the rust
    // runs from the anchor's nut, or from the brace's screws and its foot
    let (fx, fy) = ((r.f() - 0.5) * 0.03, ph / 2.0 - if bracket { 0.05 } else { 0.07 });
    let from = if bracket { fy - 0.045 } else { fy };
    let len = 0.30 + 0.18 * r.f();
    // the plane: the old decal's, on the wall's face (its back 5 mm behind the old one)
    let tex = rust_streak_tex(r.u(), pw, ph, (pw / 2.0 + fx, ph / 2.0 - from), len, if bracket { 0.013 } else { 0.009 });
    let decal = mesh(plane_geo!(pw, ph), mark_mat(&tex));
    decal.position().z = -0.004;
    grp.add(&decal);
    let name = if bracket { "Real__rustStreakBracket" } else { "Real__rustStreakAnchor" };
    let fixing = mesh(
        game::assets::props::prop_part_as_modelled(name).unwrap_or_else(|| box_geo!(0.024, 0.024, 0.027)),
        &lambert!({ map: &rust_steel_tex(), color: 0xffffff }),
    );
    fixing.position().set(fx, fy, -0.005);
    if !bracket {
        // the nut turned as it was tightened
        fixing.rotation().z = r.f() * PI;
    }
    grp.add(&fixing);
    // a mark on the wall: the placement pass leaves it where the settle put it
    grp.user().floats = true;
    g.world.add(&grp);
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

/// The rust that ran from a fixing at `from` (m from the plane's top left)
/// down a plane `pw` x `ph`, about `len` long, as wide as the fixing
/// (`w0`, half).
fn rust_streak_tex(seed: u32, pw: f64, ph: f64, from: (f64, f64), len: f64, w0: f64) -> Tex {
    let mut r = Rng(seed | 1);
    let s = r.u();
    // (sRGB) where it began, the runs, and paler where it thinned
    const DARK: [f64; 3] = [0.27, 0.13, 0.055];
    const RUST: [f64; 3] = [0.5, 0.29, 0.14];
    const PALE: [f64; 3] = [0.58, 0.42, 0.25];
    let (sx, sy) = from;
    // the runs: (offset at the top, how far it got, width, strength, seed)
    let runs: Vec<(f64, f64, f64, f64, u32)> = (0..5 + (r.f() * 5.0) as usize)
        .map(|k| {
            let o = (r.f() - 0.5) * 2.6 * w0;
            let reach = len * if k == 0 { 0.9 + 0.1 * r.f() } else { 0.25 + 0.7 * r.f() };
            (o, reach, 0.0014 + 0.003 * r.f(), 0.5 + 0.4 * r.f(), r.u())
        })
        .collect();

    decal_tex(192, 480, pw, ph, RUST, move |x, y, aa| {
        let mut px = [RUST[0], RUST[1], RUST[2], 0.0];
        let dy = y - sy;
        let t = (dy / len).clamp(0.0, 1.0);
        let shade = |t: f64| mix(mix(DARK, RUST, sstep(0.0, 0.3, t)), PALE, sstep(0.45, 1.0, t));
        // the wash: as wide as the runs spread, fading down, streaked along
        if dy > -0.004 {
            let xc = sx + (fbm(s, 0.5, y * 7.0, 2) - 0.5) * 0.02 * t;
            let hw = w0 * 1.5 + 0.042 * t.sqrt() + 0.008 * (fbm(s ^ 0x3, 1.5, y * 11.0, 2) - 0.5);
            let streak = 0.5 + 0.5 * vn(s ^ 0x9, x * 650.0, y * 7.0);
            let a = (0.6 * (1.0 - t).powf(0.7) + 0.25 * (-dy / 0.03).exp())
                * (-((x - xc) / hw).powi(2) * 1.1).exp()
                * streak
                * sstep(-0.004, 0.006, dy);
            lay(&mut px, shade(t), a);
        }
        // the runs, each darker along its edges where it dried, ending in a drop
        for &(o, reach, w, st, rs) in &runs {
            if dy < -0.002 || dy > reach + 0.01 {
                continue;
            }
            let tk = (dy / reach).clamp(0.0, 1.0);
            let xk = sx + o * (1.0 + 1.2 * tk) + (fbm(rs, 0.5, y * 9.0, 3) - 0.5) * 0.016 * tk.sqrt();
            let wk = w * (1.0 + 0.6 * tk) * (0.6 + 0.8 * fbm(rs ^ 0x5, 2.5, y * 30.0, 2));
            let along = (x - xk).abs() - wk;
            // the drop it ended in
            let end = (((x - xk) / (wk * 1.7)).hypot((dy - reach) / (wk * 2.2)) - 1.0) * wk * 1.7;
            let d = if dy < reach { along.min(end) } else { end };
            if d > aa * 2.0 {
                continue;
            }
            let inside = (0.5 - d / aa).clamp(0.0, 1.0);
            let rim = 0.65 + 0.35 * sstep(-wk * 0.9, 0.0, d);
            lay(&mut px, shade(tk * reach / len), inside * st * rim * (1.0 - 0.55 * tk));
        }
        // dark round the fixing, where the rust began, and the ring it dried to
        let n = fbm(s ^ 0x11, x * 90.0, y * 90.0, 3);
        let rr = (x - sx).hypot((y - sy) * if dy < 0.0 { 1.6 } else { 0.7 }) * (0.75 + 0.5 * n);
        let ring = w0 + 0.01;
        lay(&mut px, DARK, 0.7 * (-(rr / ring).powi(2) * 1.8).exp() * (0.7 + 0.6 * n));
        lay(&mut px, mix(DARK, RUST, 0.4), 0.22 * (-((rr - ring) / 0.004).powi(2)).exp() * sstep(0.35, 0.65, n));
        // nothing at the plane's edges
        px[3] *= sstep(0.0, 0.015, x.min(pw - x).min(y).min(ph - y));
        px
    })
}

/// Rusted steel: dark scale and orange bloom, pitted (one, shared).
fn rust_steel_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            decal_tex(64, 64, 0.03, 0.03, [0.3, 0.16, 0.08], |x, y, _| {
                let n = fbm(0x5157, x * 260.0, y * 260.0, 3);
                let mut c = mix([0.2, 0.11, 0.06], [0.48, 0.25, 0.1], sstep(0.4, 0.75, n));
                let pit = vn(0x7a11, x * 1400.0, y * 1400.0);
                c = mix(c, [0.1, 0.06, 0.04], sstep(0.75, 0.95, pit) * 0.7);
                [c[0], c[1], c[2], 1.0]
            })
        })
        .clone()
    })
}
