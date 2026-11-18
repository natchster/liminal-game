//! The vault (`docs/plan/FACILITY.md` F8; the real galleries:
//! `docs/research/facility.md` 2.2): one long chamber, a blind wall a tile
//! thick at each end, arched over at a real gallery's size (8 or 12 m
//! across, its crown 6.4-7 m up), its arch bare concrete or lined with
//! corrugated liner plate. A passage or two leads off its side, lower, to
//! the floor round it (`level::archgen::vault`). It is lit end to end: the
//! rot never puts its region out (`Game::build_regions`).
//!
//! Its one wrong thing, never said (the developer's answer): a draught with
//! no opening. The cords its lamps hang on and the cobwebs in the corners
//! where its arch springs from its walls all lean the same way, toward a
//! blind end of the gallery, never toward a door
//! (`Game::settle_vault_draughts`: moving air is the way down's call).
//!
//! Planned with the layout (`Game::plan_vaults`: the passages' low
//! ceilings, whether it is lined and which way its draught leans, on a
//! stream of its own, so no draw of the level moves); its arch laid once the
//! floor has settled (`Game::lay_vaults`); its lamps hung in `lay_panels`.

use crate::base::rng::SplitMix64;
use crate::level::archgen::{Gallery, VAULT, vault_gallery};
use crate::prelude::*;
use crate::world::arch::grime_of;
use crate::world::lining::Face;

/// A passage's ceiling off a gallery (m): under the arch's springing, so the
/// arch runs whole over its mouth
pub const PASSAGE_CEIL: f64 = 3.1;
/// The arch's rise, a share of its span (a segmental arch, meeting its walls
/// at a crease of about 30°: never tangent, `world::lining`'s lesson)
pub const RISE: f64 = 0.28;
/// How far under the zone's slab the arch's crown is (m)
pub const CROWN_UNDER: f64 = 0.03;
/// The draught's lean (rad, about 8°): its lamps' cords and its cobwebs
pub const LEAN: f64 = 0.14;
/// The share of vaults lined with liner plate
pub const LINED_SHARE: f64 = 0.6;
/// How far from the end its draught leans to a door keeps it from leaning
/// there (m)
pub const DOOR_CLEAR: f64 = 8.0;
/// Segments round an arch
const ARC_SEG: usize = 24;
/// The cobwebs' lean (rad, about 25°): lighter than a lamp, they stream
pub const WEB_LEAN: f64 = 0.44;
/// About how far apart the cobwebs hang along each springing (m)
const WEB_EVERY: f64 = 2.6;
/// The vaults' own stream
const VAULT_SALT: u64 = 0x7A0C_51E2_44D1_B093;

/// One vault of the floor
#[derive(Clone, Debug)]
pub struct Vault {
    /// its zone, and its gallery in tiles
    pub zone: usize,
    pub g: Gallery,
    /// lined with liner plate, or bare concrete
    pub lined: bool,
    /// the end its draught leans toward along the gallery (+1 its far end,
    /// -1 its near), none where both ends are open
    pub toward: Option<f64>,
    /// its arch's crown (m)
    pub crown: f64,
}

/// A vault's arch in world metres: along x or z from `lo` to `hi`, its
/// middle line across at `mid`, its half-span, its circle's radius and the
/// height of the circle's centre
#[derive(Clone, Copy, Debug)]
pub struct ArchGeom {
    pub along_x: bool,
    pub lo: f64,
    pub hi: f64,
    pub mid: f64,
    pub half: f64,
    pub r: f64,
    pub yc: f64,
}

impl ArchGeom {
    /// The arch's height over a point `s` across from its middle line (its
    /// springing beyond the walls)
    pub fn y_at(&self, s: f64) -> f64 {
        let s = s.abs().min(self.half);
        self.yc + (self.r * self.r - s * s).sqrt()
    }
    /// A world point at `u` along and `s` across, `y` up
    fn at(&self, u: f64, s: f64, y: f64) -> Vec3 {
        if self.along_x { Vec3::new(u, y, self.mid + s) } else { Vec3::new(self.mid + s, y, u) }
    }
    /// The arch's normal at angle `phi` from its crown, facing in
    fn normal(&self, phi: f64) -> Vec3 {
        let (s, c) = (phi.dsin(), phi.dcos());
        if self.along_x { Vec3::new(0.0, -c, -s) } else { Vec3::new(-s, -c, 0.0) }
    }
}

impl Game {
    /// The floor's vaults (see the module), from its layout: called with
    /// it, before any solid is built
    pub fn plan_vaults(&mut self) {
        self.vaults.clear();
        let Some(l) = self.layout.as_ref() else { return };
        let list: Vec<(usize, (i32, i32, i32, i32))> =
            l.zones.iter().enumerate().filter(|(_, z)| z.arch == VAULT).map(|(i, z)| (i, (z.x0, z.z0, z.x1, z.z1))).collect();
        let w = self.w;
        for (zi, (x0, z0, x1, z1)) in list {
            let g = vault_gallery(x0, z0, x1, z1);
            let at = |a: i32, c: i32| if g.along_x { (a, c) } else { (c, a) };
            // its passages lower than the gallery: under the arch's springing
            for tz in z0..=z1 {
                for tx in x0..=x1 {
                    let t = (tz * w + tx) as usize;
                    if self.tiles[t] == 0 && !g.has(tx, tz) {
                        self.tile_ceil[t] = PASSAGE_CEIL as f32;
                    }
                }
            }
            let mut r = SplitMix64::new(self.current_seed, VAULT_SALT ^ (zi as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
            let lined = r.next() < LINED_SHARE;
            // the draught: toward an end whose wall is whole (nothing open
            // there), the one further from its passages
            let closed = |a: i32| (g.c0..=g.c1).all(|c| self.is_wall_tile(at(a, c).0, at(a, c).1));
            let passages: Vec<i32> =
                (g.a0..=g.a1).filter(|&a| [g.c0 - 1, g.c1 + 1].iter().any(|&c| !self.is_wall_tile(at(a, c).0, at(a, c).1))).collect();
            let far = |end: i32| passages.iter().map(|&p| (p - end).abs()).min().unwrap_or(i32::MAX);
            let toward = match (closed(g.a0 - 1), closed(g.a1 + 1)) {
                (true, true) => Some(if far(g.a1) >= far(g.a0) { 1.0 } else { -1.0 }),
                (true, false) => Some(-1.0),
                (false, true) => Some(1.0),
                (false, false) => None,
            };
            let (gx, gz) = at(g.a0, g.c0);
            let crown = self.tile_ceil[(gz * w + gx) as usize] as f64 - CROWN_UNDER;
            self.vaults.push(Vault { zone: zi, g, lined, toward, crown });
        }
    }

    /// Once the floor's doors are placed: no vault's draught leans toward a
    /// door (moving air is the way down's call, `LORE.md`): one with a door
    /// within `DOOR_CLEAR` of the end it leans to leans to its other end if
    /// that is closed, or not at all
    pub fn settle_vault_draughts(&mut self) {
        let mut doors: Vec<(f64, f64)> =
            self.layout_things.iter().filter(|t| t.asset.ends_with("_door") || t.asset.ends_with("_doors")).map(|t| (t.x, t.z)).collect();
        if let Some(e) = self.exit_data.as_ref() {
            doors.push((e.pos.x, e.pos.z));
        }
        for k in 0..self.vaults.len() {
            let Some(t) = self.vaults[k].toward else { continue };
            let a = self.vault_geom(&self.vaults[k]);
            let near = |end: f64| {
                doors.iter().any(|&(x, z)| {
                    let (u, s) = if a.along_x { (x, z - a.mid) } else { (z, x - a.mid) };
                    s.abs() < a.half + 1.0 && (u - end).abs() < DOOR_CLEAR
                })
            };
            let (end, other) = if t > 0.0 { (a.hi, a.lo) } else { (a.lo, a.hi) };
            if !near(end) {
                continue;
            }
            let g = self.vaults[k].g;
            let at = |aa: i32, c: i32| if g.along_x { (aa, c) } else { (c, aa) };
            let other_end = if t > 0.0 { g.a0 - 1 } else { g.a1 + 1 };
            let closed = (g.c0..=g.c1).all(|c| self.is_wall_tile(at(other_end, c).0, at(other_end, c).1));
            self.vaults[k].toward = (closed && !near(other)).then_some(-t);
        }
    }

    /// A vault's arch in world metres
    pub fn vault_geom(&self, v: &Vault) -> ArchGeom {
        let (w, h) = (self.w as f64, self.h as f64);
        let g = &v.g;
        let (n_along, n_across) = if g.along_x { (w, h) } else { (h, w) };
        let lo = (g.a0 as f64 - n_along / 2.0) * CELL;
        let hi = (g.a1 as f64 + 1.0 - n_along / 2.0) * CELL;
        let c0 = (g.c0 as f64 - n_across / 2.0) * CELL;
        let c1 = (g.c1 as f64 + 1.0 - n_across / 2.0) * CELL;
        let half = (c1 - c0) / 2.0;
        let rise = RISE * (c1 - c0);
        let r = (rise * rise + half * half) / (2.0 * rise);
        ArchGeom { along_x: g.along_x, lo, hi, mid: (c0 + c1) / 2.0, half, r, yc: v.crown - r }
    }

    /// The vault whose gallery holds world (`x`, `z`), with its arch
    pub fn vault_at(&self, x: f64, z: f64) -> Option<(&Vault, ArchGeom)> {
        let (tx, tz) = (self.tx(x), self.tz(z));
        self.vaults.iter().find(|v| v.g.has(tx, tz)).map(|v| (v, self.vault_geom(v)))
    }

    /// Each vault's arch (see the module): one mesh a vault, its photograph
    /// mapped in metres along the gallery and round the arc; what was hung
    /// on the gallery's flat slab or high on its walls, behind the arch,
    /// taken away
    pub fn lay_vaults(&mut self) {
        if self.vaults.is_empty() {
            return;
        }
        let vaults = self.vaults.clone();
        let geoms: Vec<ArchGeom> = vaults.iter().map(|v| self.vault_geom(v)).collect();
        let mut hidden = vec![];
        for top in self.world.children() {
            if top.user().gameplay || top.user().arch {
                continue;
            }
            let mut bb = Box3::empty();
            bb.set_from_object(&top);
            if bb.is_empty() {
                continue;
            }
            let (cx, cz) = ((bb.min.x + bb.max.x) / 2.0, (bb.min.z + bb.max.z) / 2.0);
            let behind = geoms.iter().any(|a| {
                let (u, s) = if a.along_x { (cx, cz - a.mid) } else { (cz, cx - a.mid) };
                u > a.lo && u < a.hi && s.abs() < a.half && bb.min.y > a.y_at(s)
            });
            if behind {
                hidden.push(top);
            }
        }
        for top in hidden {
            self.world.remove(&top);
        }
        for v in &vaults {
            let a = self.vault_geom(v);
            let (gx, gz) = if v.g.along_x { (v.g.a0, v.g.c0) } else { (v.g.c0, v.g.a0) };
            let t = self.t_idx(gx, gz);
            let grime = grime_of(&self.region_info[self.region_map[t] as usize]);
            // (the liner plate's photograph comes with its recipe)
            let kind = if v.lined && crate::visuals::photo::photo_scale("liner").is_some() { "liner" } else { "formwork" };
            let key = format!("{kind}:{grime}");
            let [sx, sy] = self.surfaces.scale(kind);
            // its arc, a little past the walls and into the end walls, so no
            // seam shows where it meets them
            let phi0 = ((a.half + 0.01) / a.r).min(1.0).dasin();
            let (lo, hi) = (a.lo - 0.01, a.hi + 0.01);
            let steps = ((hi - lo) / CELL).ceil().max(1.0) as usize;
            let mut f = Face::default();
            for k in 0..steps {
                let u0 = lo + (hi - lo) * k as f64 / steps as f64;
                let u1 = lo + (hi - lo) * (k + 1) as f64 / steps as f64;
                for i in 0..ARC_SEG {
                    let pa = -phi0 + 2.0 * phi0 * i as f64 / ARC_SEG as f64;
                    let pb = -phi0 + 2.0 * phi0 * (i + 1) as f64 / ARC_SEG as f64;
                    let p = |u: f64, phi: f64| a.at(u, a.r * phi.dsin(), a.yc + a.r * phi.dcos());
                    let uv = |u: f64, phi: f64| (u / sx, a.r * phi / sy);
                    let q = [p(u0, pa), p(u1, pa), p(u1, pb), p(u0, pb)];
                    let ns = [a.normal(pa), a.normal(pa), a.normal(pb), a.normal(pb)];
                    let uvs = [uv(u0, pa), uv(u1, pa), uv(u1, pb), uv(u0, pb)];
                    f.quad_smooth(q, ns, uvs);
                }
            }
            let mat = self.arch_mat(&key);
            let m = f.mesh(&mat);
            m.user().arch = true;
            self.world.add(&m);
            self.lay_webs(v, &a);
        }
    }

    /// A vault's cobwebs: strands of dusty web hanging from its arch along
    /// both springings, every one leaning the same way as its lamps' cords
    /// (`WEB_LEAN`), none where its draught has no way to lean; on the
    /// vault's own stream
    fn lay_webs(&mut self, v: &Vault, a: &ArchGeom) {
        let Some(toward) = v.toward else { return };
        let mut r = SplitMix64::new(self.current_seed, VAULT_SALT ^ 0x3EB5 ^ (v.zone as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
        let mut f = Face::default();
        let lean = toward * WEB_LEAN;
        let phi0 = (a.half / a.r).min(1.0).dasin();
        for side in [-1.0, 1.0] {
            let mut u = a.lo + 0.6 + r.next() * WEB_EVERY;
            while u < a.hi - 0.6 {
                if r.next() < 0.7 {
                    // in the corner where the arch springs from its wall
                    let phi = side * (phi0 - 0.02 - r.next() * 0.06);
                    let top = a.at(u, a.r * phi.dsin() - side * 0.02, a.yc + a.r * phi.dcos() - 0.02);
                    let len = 0.3 + r.next() * 0.6;
                    let wid = 0.4 + r.next() * 0.4;
                    let (du, dy) = (len * lean.dsin(), -len * lean.dcos());
                    let along = |p: Vec3, d: f64| if a.along_x { Vec3::new(p.x + d, p.y, p.z) } else { Vec3::new(p.x, p.y, p.z + d) };
                    let bot = along(Vec3::new(top.x, top.y + dy, top.z), du);
                    let q = [along(top, -wid / 2.0), along(top, wid / 2.0), along(bot, wid / 2.0), along(bot, -wid / 2.0)];
                    // facing across the gallery, toward its middle
                    let n = if a.along_x { Vec3::new(0.0, 0.0, -side) } else { Vec3::new(-side, 0.0, 0.0) };
                    let uvs = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)];
                    f.quad_smooth(q, [n; 4], uvs);
                }
                u += WEB_EVERY * (0.6 + 0.8 * r.next());
            }
        }
        if f.idx.is_empty() {
            return;
        }
        // see-through as old web is (blended, not cut: cut, its veil was
        // a sheet and its strands scratches)
        let mat = lambert!({ map: &web_tex(), transparent: true, depthWrite: false, side: DoubleSide });
        crate::visuals::prop_look::settle_material(&mat);
        let m = f.mesh(&mat);
        m.user().arch = true;
        self.world.add(&m);
    }
}

/// Old web gone grey with dust, gathered at its top and hanging down the
/// strip (1 x 1 in uv): a veil thinning as it falls and widens, a dozen
/// strands through it; mipmapped so a far one does not shimmer (the
/// grating's lesson, `world::drops`)
fn web_tex() -> Tex {
    thread_local! {
        static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() };
    }
    T.with(|c| {
        c.get_or_init(|| {
            const W: usize = 128;
            const H: usize = 256;
            let mut px = vec![0u8; W * H * 4];
            let mut r = SplitMix64::new(0x3EB5, 0x7A0C);
            // the veil: thick under its gathering, thinning and widening down
            let ripple: Vec<f64> = (0..W).map(|_| r.next()).collect();
            for y in 0..H {
                let t = y as f64 / H as f64;
                let half = 0.12 + 0.3 * t;
                for x in 0..W {
                    let d = ((x as f64 / W as f64) - 0.5).abs() / half;
                    let a = (1.0 - t).dpowf(1.6) * (1.0 - d * d).max(0.0) * (0.55 + 0.45 * ripple[(x + y / 9) % W]);
                    let o = (y * W + x) * 4;
                    px[o..o + 4].copy_from_slice(&[92, 90, 85, (a * 150.0) as u8]);
                }
            }
            // its strands, from the gathering, wandering down and apart
            for s in 0..12 {
                let mut x = W as f64 * (0.44 + 0.12 * r.next());
                let drift = (s as f64 - 5.5) * 0.05 + (r.next() - 0.5) * 0.2;
                let end = H as f64 * (0.45 + 0.55 * r.next());
                let mut y = 0.0;
                while y < end {
                    let (ix, iy) = (x.round() as i64, y as usize);
                    for dx in [-1i64, 0, 1] {
                        let xx = ix + dx;
                        if xx >= 0 && (xx as usize) < W && iy < H {
                            let o = (iy * W + xx as usize) * 4;
                            let a = if dx == 0 { 150 } else { 60 };
                            px[o + 3] = px[o + 3].max(a);
                            px[o..o + 3].copy_from_slice(&[108, 105, 99]);
                        }
                    }
                    x += drift + (r.next() - 0.5) * 0.7;
                    y += 1.0;
                }
            }
            let t = Tex::data_rgba(px, W as u32, H as u32);
            {
                let mut tt = t.t();
                tt.generate_mipmaps = true;
                tt.min_filter = Filter::LinearMipmapLinear;
                tt.mag_filter = Filter::Linear;
            }
            t
        })
        .clone()
    })
}
