//! `place_wall_water_streaks`
use game::prelude::*;
use placers_kit::noise::fbm;
use placers_kit::noise::smin;
use placers_kit::noise::sstep;
use placers_kit::noise::tide;
use placers_kit::noise::vn as vnoise;
use placers_kit::paint::mark_mat;
use placers_kit::paint::mark_tex;
use placers_kit::paint::paint_px_indexed as paint_px;
use placers_kit::place::mount_slid_ref as wall_mount;
use placers_kit::place::shuffle;
use placers_kit::place::wall_candidates;
use placers_kit::wet_floor::floor_kind;
use placers_kit::wet_floor::Px;

// Wall water streak: 20% chance per level, 1-2 streaks. Where water has
// come down a wall from the ceiling's edge, again and again, and dried
// (`docs/research/props-remodelling/marks_on_the_rooms_surfaces.md`): it runs straight down from where it came through, narrow
// there and wider below, splitting into runs that end in drips; the edge of
// each run is a darker deposit (a tide line), its middle a fainter wash of
// what it carried: grey-brown on paint and paper, a dark damp run with a
// white salt crust on concrete and brick, a faint rust on glazed tile.
// Painted on a transparent plane from the ceiling's edge to where the old
// streak ended.
pub(crate) fn place_wall_water_streaks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.80 {
        return;
    } // 20% chance
    let count = 1 + (rnd() * 2.0).floor() as usize;
    let mut candidates = wall_candidates(g);
    shuffle(&mut candidates);
    let mut i = 0;
    while i < count && i < candidates.len() {
        let c = candidates[i];
        let surface = wall_surface(g, c.x, c.z);
        // the old streak's draws: its width (how far its runs spread, 6-10
        // cm) and its length (0.8-1.8 m below its old top, 10 cm under the
        // ceiling). Its plane's middle is where the old one's was and it is
        // as wide, or 8 cm where it was narrower (`settle_props` probes the
        // wall's face from there, across the width when it is 8 cm or more:
        // so it finds what it found); it reaches up to the ceiling and as
        // far again below its foot
        let streak_w = 0.06 + rnd() * 0.04;
        let streak_h = 0.8 + rnd() * 1.0;
        let streak_top = WALL_H - 0.10;
        let up = WALL_H - 0.005 - streak_top;
        let streak = streak_mesh(streak_w.max(STREAK_W), streak_h + 2.0 * up, streak_h + up, surface, true);
        let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.008);
        let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.008);
        wall_mount(&streak, &c, wall_x, wall_z, streak_top - streak_h / 2.0, 1.4);
        g.world.add(&streak);
        // 25% chance: where it dripped and stood, a dried ring on the
        // floor below (its old radius 9-16 cm, 1.3 × 0.85 of it)
        if rnd() < 0.25 {
            // (on the old disc, 1.3 × 0.85 of its radius: its outline the
            // old one's, the ring painted inside it)
            let r = 0.09 + rnd() * 0.07;
            let floor = floor_kind(g, c.x, c.z);
            let fs = mesh(circle_geo!(r, 12), &foot_ring_mat(r, floor));
            fs.rotation().x = -PI / 2.0;
            fs.rotation().z = rnd() * PI * 2.0;
            fs.scale().set(1.3, 0.85, 1);
            // just below the streak, out from the wall
            let sp = streak.pos();
            fs.position().set(sp.x - c.dx as f64 * 0.14, 0.004, sp.z - c.dz as f64 * 0.14);
            g.world.add(&fs);
        }
        // 25% chance: 1-2 more runs close by, from the same edge
        if rnd() < 0.25 {
            let sib_count = 1 + (rnd() * 2.0).floor() as i32;
            for _sib in 0..sib_count {
                // the old sibling's draws: width, length, side, offset, drop
                let sib_w = 0.04 + rnd() * 0.04;
                let sib_h = 0.5 + rnd() * 0.7;
                let sib_off = (if rnd() < 0.5 { -1.0 } else { 1.0 }) * (0.10 + rnd() * 0.10);
                let drop = rnd() * 0.12;
                // from the ceiling's edge to where the old one ended (its
                // middle and width as the old one's)
                let up = WALL_H - 0.005 - (streak_top - drop);
                let sib = streak_mesh(sib_w.max(STREAK_W), sib_h + 2.0 * up, sib_h + up, surface, false);
                sib.rotation().y = streak.rot().y;
                let sp = streak.pos();
                let y = streak_top - drop - sib_h / 2.0;
                if c.dx.abs() == 1 {
                    sib.position().set(sp.x, y, sp.z + sib_off);
                } else {
                    sib.position().set(sp.x + sib_off, y, sp.z);
                }
                g.world.add(&sib);
            }
        }
        i += 1;
    }
}

/// The narrowest a streak's plane is: just under the width (8 cm) past which
/// `settle_props` probes a wall's face at its sides as well as its middle
const STREAK_W: f64 = 0.0799;

/// What the wall seen from open tile (x, z) is: 0 paper or paint (wallpaper,
/// drywall), 1 masonry (concrete, brick), 2 glazed tile
fn wall_surface(g: &Game, x: i32, z: i32) -> u8 {
    if g.region_flags_at(x, z) & game::visuals::lighting::RF_MOSS != 0 {
        return 0;
    }
    match g.zone_ref(x, z).map(|zn| zn.wall_mat) {
        Some("concrete" | "brick") => 1,
        Some("tile") => 2,
        _ => 0,
    }
}

/// The white salts water leaves on concrete and brick (sRGB)
const SALT: [f64; 3] = [0.78, 0.77, 0.72];

/// One run of a water streak, in its plane's metres (x across, y down from
/// the ceiling's edge): where it starts, how it leans, where it ends, its
/// half width at its start and at its widest, how much it narrows again
/// toward its foot, its noise
struct Run {
    xs: f64,
    lean: f64,
    y0: f64,
    y1: f64,
    w0: f64,
    w1: f64,
    taper: f64,
    s: u32,
}

impl Run {
    /// Its middle at depth `y`: straight down, wavering a little as it goes
    fn centre(&self, y: f64) -> f64 {
        let wander = (fbm(self.s, y * 3.0, 0.5, 3) - 0.5) * 0.016 + (vnoise(self.s ^ 0x5bd1, y * 40.0, 0.5) - 0.5) * 0.0025;
        self.xs + self.lean * (y - self.y0) + wander * ((y - self.y0) / 0.1).clamp(0.0, 1.0)
    }
    /// Its half width at depth `y`: narrow where it starts, wider below as
    /// it slowed and spread, narrowing again to the drop it ended in
    fn half(&self, y: f64) -> f64 {
        let t = ((y - self.y0) / (self.y1 - self.y0)).clamp(0.0, 1.0);
        let grow = self.w0 + (self.w1 - self.w0) * (t / 0.45).min(1.0).powf(0.6);
        grow * (1.0 - self.taper * sstep(0.45, 1.0, t)) * (0.6 + 0.8 * fbm(self.s ^ 0x2c1b, y * 7.0, 2.5, 2))
    }
}

/// A dried run of dirty water down a wall from the ceiling's edge (the top
/// of a plane `w × ph` m facing +z), its runs spreading across it to about
/// `w` and the longest ending `foot` m down; on `surface` (`wall_surface`);
/// `main`: the first of a group, with more runs
fn streak_mesh(w: f64, ph: f64, foot: f64, surface: u8, main: bool) -> Obj {
    let pw = w;
    let (tw, th) = (128.0, (ph / 0.003).round().clamp(160.0, 640.0));
    // (sRGB) its tide lines, the wash inside them, and how strong each is
    let (line, fill, k_line, k_fill) = match surface {
        1 => ([0.12, 0.115, 0.105], [0.17, 0.165, 0.15], 0.42, 0.56),
        2 => ([0.45, 0.34, 0.22], [0.55, 0.46, 0.35], 0.38, 0.26),
        _ => ([0.29, 0.22, 0.14], [0.38, 0.31, 0.21], 0.45, 0.52),
    };
    let tex = mark_tex(tw, th, line, move |ctx, cw, ch| {
        let seed = (rnd() * 4.0e9) as u32;
        let (sx, sy) = (pw / cw, ph / ch);
        let aa = sx.max(sy * 0.5);
        // where it came through at the ceiling's edge, spread a little along it
        let x0 = pw * (0.45 + 0.1 * rnd());
        let (src_rx, src_ry) = (w * (0.32 + 0.14 * rnd()), 0.035 + 0.05 * rnd());
        // the runs: side by side from there, the first to the old foot, the
        // rest shorter and fainter, some branching off lower down
        let n = if main { 3 + (rnd() * 3.0) as usize } else { 2 + (rnd() * 2.0) as usize };
        let mut runs: Vec<(Run, f64)> = vec![];
        for k in 0..n {
            let s = seed.wrapping_add(1 + k as u32 * 7919);
            let (y0, xs) = if k == 0 || rnd() < 0.55 {
                (0.0, (x0 + if k == 0 { 0.0 } else { (rnd() - 0.5) * 2.2 * src_rx }).clamp(0.008, pw - 0.008))
            } else {
                let y0 = ph * (0.05 + 0.3 * rnd());
                (y0, runs[0].0.centre(y0))
            };
            let y1 = if k == 0 { foot - 0.01 * rnd() } else { y0 + (foot - y0) * (0.25 + 0.65 * rnd()) };
            let lean = (rnd() - 0.5) * w * 0.2 / ph;
            // a few millimetres wide where it set off, wider as it slowed:
            // the first a broad wet run, the rest rivulets
            let w0 = (0.002 + 0.002 * rnd()) / (1.0 + 0.3 * k as f64);
            let w1 = if k == 0 { w * (0.2 + 0.1 * rnd()) } else { (0.003 + 0.006 * rnd()) / (1.0 + 0.2 * k as f64) };
            let taper = if k == 0 { 0.5 + 0.2 * rnd() } else { 0.1 + 0.2 * rnd() };
            let strength = if k == 0 { 1.0 } else { 0.4 + 0.5 * rnd() };
            runs.push((Run { xs, lean, y0, y1, w0, w1, taper, s }, strength));
        }
        // the drop each ended in, dried a little fuller than its trail:
        // (x, y, half width, half length)
        let tips: Vec<(f64, f64, f64, f64)> = runs
            .iter()
            .map(|(r, _)| {
                let rb = r.half(r.y1) * 1.2 + 0.0008;
                (r.centre(r.y1 - rb * 1.4), r.y1 - rb * 1.4, rb, rb * 1.4)
            })
            .collect();
        let rows: Vec<Vec<(f64, f64)>> = (0..ch as usize)
            .map(|j| {
                let y = (j as f64 + 0.5) * sy;
                runs.iter().map(|(r, _)| (r.centre(y), r.half(y))).collect()
            })
            .collect();
        paint_px(ctx, cw, ch, |i, j| {
            let (x, y) = ((i as f64 + 0.5) * sx, (j as f64 + 0.5) * sy);
            let wob = (vnoise(seed, x * 260.0, y * 90.0) - 0.5) * 0.0014 + (vnoise(seed ^ 0x77, x * 60.0, y * 14.0) - 0.5) * 0.004;
            // the wetted patch it came from, under the ceiling
            let src = (((x - x0) / src_rx).hypot(y / src_ry) - 1.0) * src_ry + wob;
            let (mut fill_a, mut line_a, mut near) = (0.0f64, 0.0f64, f64::MAX);
            let mut add = |d: f64, strength: f64, fade: f64| {
                near = near.min(d);
                if d > 0.006 {
                    return;
                }
                let inside = (0.5 - d / aa).clamp(0.0, 1.0);
                // streaked down the way it ran, thinner in places
                let streaks = 0.5 + 0.5 * vnoise(seed ^ 0xabc, x * 380.0, y * 7.0);
                let f = inside * k_fill * strength * fade * streaks * (0.7 + 0.3 * vnoise(seed ^ 0x3c3c, x * 40.0, y * 12.0));
                let t0 = tide(d, aa, 0.003);
                let t1 = tide(d + 0.0028 + (vnoise(seed ^ 0x1f, x * 150.0, y * 40.0) - 0.5) * 0.002, aa, 0.0018) * 0.4;
                let l = (1.0 - (1.0 - t0) * (1.0 - t1))
                    * k_line
                    * strength
                    * (0.6 + 0.4 * fade)
                    * (0.4 + 0.6 * vnoise(seed ^ 0x6e, x * 30.0, y * 5.0));
                fill_a = 1.0 - (1.0 - fill_a) * (1.0 - f);
                line_a = 1.0 - (1.0 - line_a) * (1.0 - l);
            };
            add(src, 1.0, 1.0);
            for (((r, strength), &(xc, hw)), &(bx, by, bw, bh)) in runs.iter().zip(&rows[j]).zip(&tips) {
                let along = if y < r.y0 {
                    (x - r.xs).hypot(y - r.y0) - r.w0
                } else if y <= by {
                    (x - xc).abs() - hw
                } else {
                    1.0
                };
                let tip = (((x - bx) / bw).hypot((y - by) / bh) - 1.0) * bw;
                let d = smin(along, tip, 0.002) + wob;
                // fainter as it ran thin, darker again where the drop dried
                let t = ((y - r.y0) / (r.y1 - r.y0)).clamp(0.0, 1.0);
                let fade = 1.1 - 0.5 * t + 0.4 * sstep(by - bh, by, y);
                add(d, *strength, fade);
            }
            let mut px = Px::CLEAR;
            // the faint wetting round them all, soft-edged
            let halo = sstep(0.018, 0.0, near + (vnoise(seed ^ 0x4a, x * 40.0, y * 8.0) - 0.5) * 0.012);
            px.lay(fill, halo * k_fill * 0.22);
            px.lay(fill, fill_a);
            px.lay(line, line_a);
            if surface == 1 {
                // the salts it left: a soft white bloom along its edges and
                // where the drops dried, in patches
                let bloom = sstep(0.45, 0.72, fbm(seed ^ 0x99, x * 110.0, y * 22.0, 3));
                let crust = ((line_a * 1.3 + fill_a * 0.6) * bloom * 0.75).min(0.6);
                px.lay(SALT, crust);
            }
            // (and nothing at the plane's sides)
            let side = sstep(0.0, 0.004, x.min(pw - x));
            [px.c[0], px.c[1], px.c[2], px.a * side]
        });
    });
    mesh(plane_geo!(pw, ph), &mark_mat(&tex))
}

/// The dried ring where water dripped and stood on a floor of `floor`
/// (`floor_kind`), for a disc of radius `r` (`circle_geo!`, its UVs the
/// square round it) scaled 1.3 × 0.85: an irregular ellipse inside the
/// scaled disc, a tide line at its rim and older ones inside it, its middle
/// paler (painted as it will look once scaled)
fn foot_ring_mat(r: f64, floor: u8) -> Mat {
    let (rx, rz) = (r * 1.3 * 0.78, r * 0.85 * 0.78);
    let (pw, ph) = (r * 2.0 * 1.3, r * 2.0 * 0.85);
    let (tw, th) = (160.0, 160.0);
    let (line, fill, k_line, k_fill) = match floor {
        1 => ([0.15, 0.145, 0.135], [0.22, 0.21, 0.2], 0.45, 0.2),
        2 => ([0.45, 0.38, 0.28], [0.6, 0.55, 0.47], 0.4, 0.1),
        _ => ([0.2, 0.15, 0.09], [0.3, 0.24, 0.16], 0.55, 0.18),
    };
    let tex = mark_tex(tw, th, line, move |ctx, cw, ch| {
        let seed = (rnd() * 4.0e9) as u32;
        let (sx, sy) = (pw / cw, ph / ch);
        let aa = sx.max(sy) * 1.1;
        let r0 = rx.min(rz);
        // each wetting's rim: (how far inside the last, how strong, its noise)
        let rings: Vec<(f64, f64, u32)> = (0..2 + (rnd() * 2.0) as usize)
            .map(|k| {
                let (o, s) = if k == 0 { (0.0, 1.0) } else { (0.08 + 0.3 * rnd(), 0.35 + 0.4 * rnd()) };
                (o, s, seed.wrapping_add(k as u32 * 977 + 1))
            })
            .collect();
        paint_px(ctx, cw, ch, |i, j| {
            let (x, y) = ((i as f64 + 0.5) * sx - pw / 2.0, (j as f64 + 0.5) * sy - ph / 2.0);
            let (u, v) = (x / rx, y / rz);
            let rho = u.hypot(v);
            if rho > 1.45 {
                return [line[0], line[1], line[2], 0.0];
            }
            let (cu, cv) = (u / rho.max(1e-6), v / rho.max(1e-6));
            let fine = (vnoise(seed ^ 0x51, x * 160.0, y * 160.0) - 0.5) * 0.0018;
            let (mut d0, mut lines) = (0.0, 0.0);
            for (k, &(o, s, rs)) in rings.iter().enumerate() {
                let lobe = 1.0
                    + 0.3 * (fbm(rs, cu * 1.3 + 7.0, cv * 1.3 + 7.0, 3) - 0.5)
                    + 0.07 * (vnoise(rs ^ 0x33, cu * 5.0 + 3.0, cv * 5.0 + 3.0) - 0.5);
                let d = (rho - (1.0 - o) * lobe) * r0 + fine;
                if k == 0 {
                    d0 = d;
                }
                lines = 1.0 - (1.0 - lines) * (1.0 - tide(d, aa, 0.004) * s);
            }
            let mut px = Px::CLEAR;
            let inside = (0.5 - d0 / aa).clamp(0.0, 1.0);
            let pale = 0.55 + 0.45 * sstep(0.0, 0.8, rho);
            px.lay(fill, inside * k_fill * pale * (0.7 + 0.3 * vnoise(seed ^ 0x3c, x * 40.0, y * 40.0)));
            px.lay(line, lines * k_line);
            if floor == 1 {
                px.lay(SALT, (lines * (vnoise(seed ^ 0x99, x * 300.0, y * 300.0) - 0.45).max(0.0) * 1.5).min(0.5));
            }
            px.rgba()
        });
    });
    mark_mat(&tex)
}
