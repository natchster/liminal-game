//! `place_peeling_wallpapers`
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
use placers_kit::place::wall_key_at;

// Peeling wallpaper (`docs/research/props-remodelling/wall_marks_and_wear.md`): the
// corner of a drop come away at its seam, lifted off the wall, turned over
// and hanging, the paper's pale back to the room; behind it the bare wall
// it was pasted to, stained with the old paste, a pencil plumb line by the
// seam, scraps of the paper's backing still stuck; its torn edges white with
// the paper's core. On a papered wall it is the wall's own paper; on any
// other, a scrap of old paper left on it, torn round, peeling the same way.
pub(crate) fn place_peeling_wallpapers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    let wallpaper = group();
    let (pw, ph) = (0.30, 0.50);
    // which drop came away, the one to the left of the seam or to the right
    // (the old peeled corner's side)
    let side = if rnd() < 0.5 { -1.0 } else { 1.0 };
    mount_on_wall(g, &wallpaper, &c, 0.005, 1.4, 0.5);
    // its layout, from where it is (no draws)
    let p = wallpaper.pos();
    let mut r = Rng(seed_of(&[p.x, p.y, p.z, c.dx as f64, c.dz as f64]));
    let key = wall_key_at(g, c.x, c.z);
    let papered = key.starts_with("wp:");
    // the seam (x, along the wall) and the line the paper still holds along
    let seam = -side * (0.005 + 0.025 * r.f());
    let foot = -0.03 + 0.05 * r.f();
    let tex = peeling_wallpaper_tex(r.u(), pw, ph, side, seam, foot, papered);
    let decal = mesh(plane_geo!(pw, ph), mark_mat(&tex));
    decal.position().z = -0.004;
    wallpaper.add(&decal);
    // the corner come away: its printed face (the wall's own paper, or the
    // scrap's), its back
    let face_mat = if papered { g.arch_mat(&key).duplicate() } else { lambert!({ color: OLD_PAPER_HEX }) };
    let back_mat = lambert!({ map: &paper_back_tex(), color: 0xffffff });
    let hand = if side > 0.0 { "R" } else { "L" };
    for (part, mat) in [("Front", &face_mat), ("Back", &back_mat)] {
        let name = format!("Real__peelingWallpaperCurl{hand}{part}");
        let geo = game::assets::props::prop_part_as_modelled(&name).unwrap_or_else(|| box_geo!(0.12, 0.1, 0.07));
        let m = mesh(geo, mat);
        m.position().set(seam, foot, -0.005);
        wallpaper.add(&m);
    }
    // a mark on the wall: the placement pass leaves it where the settle put it
    wallpaper.user().floats = true;
    g.world.add(&wallpaper);
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

/// An old wallpaper's ground: cream gone yellow (the scrap's, and its curl's face)
const OLD_PAPER: [f64; 3] = [0.77, 0.71, 0.56];

const OLD_PAPER_HEX: u32 = 0xc4b58f;

/// The wall where a drop's corner came away, across a plane `pw` x `ph`
/// (x right from its middle, y up from its middle, as the group is): the
/// seam at `seam`, the drop on `side` of it, pasted still below `foot`;
/// on a `papered` wall the paper round it is the wall's own (clear here),
/// else a scrap of old paper is painted round it.
fn peeling_wallpaper_tex(seed: u32, pw: f64, ph: f64, side: f64, seam: f64, foot: f64, papered: bool) -> Tex {
    let mut r = Rng(seed | 1);
    let s = r.u();
    // (sRGB) plaster, the old paste on it, the paper's backing left stuck,
    // the paper's white core at a tear, pencil
    const PLASTER: [f64; 3] = [0.64, 0.61, 0.55];
    const PASTE: [f64; 3] = [0.62, 0.5, 0.31];
    const BACKING: [f64; 3] = [0.84, 0.81, 0.74];
    const CORE: [f64; 3] = [0.9, 0.88, 0.82];
    const SHADE: [f64; 3] = [0.05, 0.045, 0.04];
    const STRIPE: [f64; 3] = [0.6, 0.5, 0.38];
    let (cw, cl) = (0.12, 0.19);
    // the scrap's torn outline: its half width and half height, ragged
    let (sw, sh) = (0.1 + 0.02 * r.f(), 0.17 + 0.03 * r.f());
    let stripe_at = r.f() * 0.04;

    decal_tex(288, 480, pw, ph, if papered { PLASTER } else { OLD_PAPER }, move |cx, cy, aa| {
        // the group's frame: x right, y up, from the middle
        let (x, y) = (cx - pw / 2.0, ph / 2.0 - cy);
        let mut px = [PLASTER[0], PLASTER[1], PLASTER[2], 0.0];
        let u = (x - seam) * side; // across the drop from the seam
                                   // the bare patch: from the seam to the torn far side, from the foot
                                   // to the torn top; how far inside it (m, < 0 outside)
        let far = cw + 0.006 * (fbm(s ^ 0x21, y * 60.0, 0.5, 3) - 0.5) * 2.0 + 0.002 * (vn(s ^ 0x22, y * 700.0, 0.5) - 0.5);
        let top = foot + cl + 0.01 * (fbm(s ^ 0x23, x * 45.0, 0.5, 3) - 0.5) * 2.0 + 0.002 * (vn(s ^ 0x24, x * 700.0, 0.5) - 0.5);
        let inside = u.min(far - u).min(y - foot).min(top - y);
        // the scrap round it (another wall): its torn outline
        let scrap = if papered {
            1.0
        } else {
            // a rounded, torn outline: bitten into deeply here and there
            let (u, v) = (x / sw, y / sh);
            let q = (u.abs().powf(2.6) + v.abs().powf(2.6)).powf(1.0 / 2.6);
            let bite = 0.28 * (fbm(s ^ 0x31, x * 9.0, y * 9.0, 3) - 0.5) * 2.0 + 0.05 * (fbm(s ^ 0x32, x * 60.0, y * 60.0, 2) - 0.5);
            (1.0 - q + bite) * sw.min(sh)
        };
        if !papered && scrap > -0.003 && inside < 0.0 {
            // the old paper: faded stripes in pairs and a small sprig between,
            // yellowed, foxed, grimier low
            let xs = (x - stripe_at).rem_euclid(0.04);
            let st = (-((xs - 0.004) / 0.0007).powi(2)).exp() + (-((xs - 0.009) / 0.0007).powi(2)).exp();
            let (gi, gj) = (((x - stripe_at) / 0.04 - 0.5).floor(), (y / 0.05).floor());
            let (sx0, sy0) = ((gi + 0.5 + 0.5) * 0.04 + stripe_at, (gj + 0.5) * 0.05 + if (gi as i32) % 2 == 0 { 0.0 } else { 0.025 });
            let sprig = (0.5 - (((x - sx0) * 1.4).hypot(y - sy0 - 0.0) - 0.0035) / aa).clamp(0.0, 1.0);
            let fox = sstep(0.62, 0.8, fbm(s ^ 0x41, x * 40.0, y * 40.0, 3));
            let mut c = mix(OLD_PAPER, STRIPE, st * 0.55 + sprig * 0.4);
            c = mix(c, [0.55, 0.42, 0.27], fox * 0.35 + 0.1 * sstep(0.0, 0.2, -y));
            px = [c[0], c[1], c[2], (0.5 + scrap / aa).clamp(0.0, 1.0)];
            // its torn edge: the white core of the paper
            let fr = 0.0012 + 0.0012 * vn(s ^ 0x42, x * 300.0, y * 300.0);
            let a = px[3];
            lay(&mut px, CORE, sstep(fr, 0.0, scrap) * 0.85 * a);
        }
        // the paste left on the scrap's wall round it
        if !papered && scrap > -0.012 && scrap < 0.0 {
            lay(&mut px, PASTE, 0.25 * sstep(-0.012, 0.0, scrap) * fbm(s ^ 0x43, x * 50.0, y * 12.0, 2));
        }
        if inside > -0.004 {
            let fill = (0.5 + inside / aa).clamp(0.0, 1.0);
            // the wall it was pasted to: plaster on a papered wall, the
            // wall's own face on another
            if papered {
                let m = 0.92 + 0.12 * fbm(s ^ 0x51, x * 90.0, y * 90.0, 3);
                lay(&mut px, [PLASTER[0] * m, PLASTER[1] * m, PLASTER[2] * m], fill);
            }
            // the paste, in the brush's strokes, and stains where it soaked
            let stroke = fbm(s ^ 0x52, x * 70.0 + y * 8.0, y * 9.0, 3);
            lay(&mut px, PASTE, fill * (0.15 + 0.4 * sstep(0.45, 0.75, stroke)));
            // scraps of the paper's backing torn off and left stuck, mostly at the edges
            let edge = sstep(0.03, 0.0, inside);
            let back = fbm(s ^ 0x53, x * 24.0, y * 24.0, 4) + 0.2 * edge;
            let bk = sstep(0.6, 0.62, back);
            lay(&mut px, BACKING, fill * bk * 0.92);
            // and the white of its torn edge
            lay(&mut px, CORE, fill * 0.6 * (bk * (1.0 - bk) * 4.0).min(1.0));
            // the pencil line the paperhanger hung the drop to, by the seam
            let pl = (u - 0.0025).abs();
            lay(
                &mut px,
                [0.42, 0.42, 0.44],
                fill * 0.35 * (0.5 - (pl - 0.0003) / aa).clamp(0.0, 1.0) * sstep(0.3, 0.6, vn(s ^ 0x54, 0.5, y * 40.0)),
            );
            // the shadow the lifted corner throws in, under its turn
            let sh = sstep(0.08, 0.0, y - foot) * sstep(0.0, 0.015, u) * sstep(0.0, 0.015, far - u);
            lay(&mut px, SHADE, fill * 0.3 * sh);
        }
        // the paper's torn edge round the patch: the white core, a hair proud
        if inside < 0.0 && inside > -0.004 && u > -0.001 {
            let fr = 0.0011 + 0.0016 * vn(s ^ 0x55, x * 260.0, y * 260.0);
            let k = sstep(fr, 0.0, -inside);
            // not along the seam (the next drop's cut edge) nor the foot (the turn)
            let torn = sstep(0.002, 0.006, u) * sstep(0.0, 0.004, y - foot);
            lay(&mut px, CORE, k * torn * 0.85);
        }
        // the seam: the next drop's cut edge, lifted a hair: a lit lip and its shadow
        let ds = (x - seam) * side;
        let near = sstep(0.0, 0.07, ph / 2.0 - y.abs()) * if papered { 1.0 } else { sstep(-0.003, 0.003, scrap) };
        lay(&mut px, SHADE, 0.35 * near * (0.5 - (ds - 0.0008).abs() / 0.0008).clamp(0.0, 1.0));
        lay(&mut px, CORE, 0.25 * near * (0.5 - (ds + 0.0006).abs() / 0.0006).clamp(0.0, 1.0));
        // under the lifted corner's hanging end, the wall in its shade
        let hang = sstep(0.0, 0.02, u) * sstep(0.0, 0.02, cw - u) * (-((y - (foot - 0.05)) / 0.05).powi(2)).exp();
        lay(&mut px, SHADE, 0.22 * hang);
        // nothing at the plane's edges
        px[3] *= sstep(0.0, 0.01, (pw / 2.0 - x.abs()).min(ph / 2.0 - y.abs()));
        px
    })
}

/// The back of a wallpaper: off-white paper, the paste dried on it in
/// strokes, grey with the wall's dust (one, shared).
fn paper_back_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            decal_tex(128, 128, 0.12, 0.12, [0.8, 0.77, 0.68], |x, y, _| {
                let stroke = fbm(0x9a9e, x * 60.0 + y * 6.0, y * 10.0, 3);
                let mut c = mix([0.75, 0.71, 0.61], [0.64, 0.54, 0.37], 0.6 * sstep(0.4, 0.8, stroke));
                c = mix(c, [0.6, 0.6, 0.58], 0.25 * sstep(0.55, 0.8, fbm(0x9a9f, x * 140.0, y * 140.0, 2)));
                [c[0], c[1], c[2], 1.0]
            })
        })
        .clone()
    })
}
