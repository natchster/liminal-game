//! The Habitable Zone as a column hall (`docs/plan/FACILITY.md` F6; the real
//! halls: `docs/research/facility.md`): its walls and columns painted
//! two-tone, dark to the hip and pale above (`twotone`, `visuals::photo`;
//! `world::lining::wall_mat_of`), each free-standing column flaring into
//! the slab through a capital and a drop panel as a flat slab's columns do,
//! under a bare slab cast against steel forms (`lining::ceil_mat_of`). Its
//! one wrong thing stays its puddles with no source (`LORE.md`), as the wet
//! floors lay them: larger sheets of standing water were tried and read as
//! decals with no light in them (the blind review, twice).
//!
//! Architecture, made once the floor has settled (`Game::lay_halls`, after
//! the lining): from the solids as they are, with no random draw. None where
//! the moss has papered a hall (the Lobby's look).

use crate::level::archgen::{BOX_FLOOR, BOX_PILLAR, BOX_VOID};
use crate::prelude::*;
use crate::visuals::lighting::RF_MOSS;
use crate::world::arch::ARCH_CHUNK;
use crate::world::lining::Face;
use std::collections::HashMap;

/// The archetype these are the Habitable Zone's of
pub const HALL: &str = "hall";
/// A capital: how far below the slab it starts flaring (m), how far out from
/// the column's face it flares (m), and its drop panel's depth and how far it
/// reaches out from the column's face (m)
pub const CAPITAL_H: f64 = 0.95;
pub const CAPITAL_OUT: f64 = 0.55;
pub const PANEL_T: f64 = 0.2;
pub const PANEL_OUT: f64 = 0.95;
/// Under this a hall's slab is too low for a capital (m)
pub const CAPITAL_MIN_CEIL: f64 = 5.0;

impl Game {
    /// The floor tile `t` is a bare hall's (not papered by the moss)
    fn bare_hall(&self, t: usize) -> bool {
        let Some(l) = self.layout.as_ref() else { return false };
        l.zones[l.zone_of[t] as usize].arch == HALL && self.region_info[self.region_map[t] as usize].flags & RF_MOSS == 0
    }

    /// The Habitable Zone's capitals (see the module)
    pub fn lay_halls(&mut self) {
        self.lay_capitals();
    }

    /// A capital and a drop panel on each free-standing column of a bare
    /// hall whose eight neighbours are its floor under one slab
    fn lay_capitals(&mut self) {
        let Some(s) = self.solids.take() else { return };
        let (w, h) = (self.w, self.h);
        let mut cols: Vec<(String, f64, f64, f64, f64, f64)> = vec![];
        for bi in 0..s.count {
            let b = |k: usize| s.boxes[bi * 8 + k] as f64;
            if b(7) as i32 != BOX_PILLAR || b(4) > 0.01 {
                continue;
            }
            let owner = b(6) as usize;
            let (ox, oz) = ((owner as i32) % w, (owner as i32) / w);
            // its neighbours: a bare hall's floor, one slab, tall enough
            let Some(top) = (|| {
                let mut ceil = None;
                for dz in -1..=1 {
                    for dx in -1..=1 {
                        if dx == 0 && dz == 0 {
                            continue;
                        }
                        let (nx, nz) = (ox + dx, oz + dz);
                        if nx < 0 || nz < 0 || nx >= w || nz >= h {
                            return None;
                        }
                        let t = (nz * w + nx) as usize;
                        if self.tiles[t] != 0 || !self.bare_hall(t) {
                            return None;
                        }
                        let c = self.tile_ceil[t] as f64;
                        match ceil {
                            None => ceil = Some(c),
                            Some(c0) if (c0 - c).abs() > 1e-3 => return None,
                            _ => {}
                        }
                    }
                }
                ceil
            })() else {
                continue;
            };
            if top < CAPITAL_MIN_CEIL || b(5) < top - 0.01 {
                continue;
            }
            // nothing else where its flare and panel would be (a pier, a
            // lintel, a jamb beside it: the blind review saw one run into a
            // pier's top)
            let reach = (PANEL_OUT + 0.02) / CELL;
            let (u0, v0, u1, v1) = (b(0) - reach, b(1) - reach, b(2) + reach, b(3) + reach);
            let clear = (-1..=1).all(|dz| {
                (-1..=1).all(|dx| {
                    let t = ((oz + dz) * w + ox + dx) as usize;
                    (s.start[t]..s.start[t + 1]).all(|kk| {
                        let c = s.items[kk as usize] as usize;
                        let q = |k: usize| s.boxes[c * 8 + k] as f64;
                        c == bi
                            || matches!(q(7) as i32, BOX_VOID | BOX_FLOOR)
                            || q(5) < top - CAPITAL_H
                            || q(2) <= u0
                            || q(0) >= u1
                            || q(3) <= v0
                            || q(1) >= v1
                    })
                })
            });
            if !clear {
                continue;
            }
            let key = self.wall_mat_key(owner);
            let (x0, z0, x1, z1) = (
                (b(0) - w as f64 / 2.0) * CELL,
                (b(1) - h as f64 / 2.0) * CELL,
                (b(2) - w as f64 / 2.0) * CELL,
                (b(3) - h as f64 / 2.0) * CELL,
            );
            cols.push((key, (x0 + x1) / 2.0, (z0 + z1) / 2.0, (x1 - x0) / 2.0, (z1 - z0) / 2.0, top));
        }
        self.solids = Some(s);
        let mut faces: Vec<(String, (i64, i64), Face)> = vec![];
        let mut fi: HashMap<(String, (i64, i64)), usize> = HashMap::new();
        for (key, cx, cz, hx, hz, top) in cols {
            let kind = key.split(':').next().unwrap_or("");
            let [sx, sy] = self.surfaces.scale(kind);
            let chunk = ((self.to_u(cx) / ARCH_CHUNK).floor() as i64, (self.to_v(cz) / ARCH_CHUNK).floor() as i64);
            let k = *fi.entry((key.clone(), chunk)).or_insert_with(|| {
                faces.push((key.clone(), chunk, Face::default()));
                faces.len() - 1
            });
            capital_face(&mut faces[k].2, cx, cz, hx, hz, top, (sx, sy));
        }
        for (key, _, f) in faces {
            let mat = self.arch_mat(&key);
            let m = f.mesh(&mat);
            m.user().arch = true;
            self.world.add(&m);
        }
    }
}

/// One column's capital and drop panel laid into `f`: the column's middle
/// (`cx`, `cz`) and half-widths, the slab at `top`. Its paint mapped as its
/// column's walls' is, by height (the pale above the band) and along each
/// face; the panel's underside along x and, by a little height, along z.
fn capital_face(f: &mut Face, cx: f64, cz: f64, hx: f64, hz: f64, top: f64, scale: (f64, f64)) {
    let y0 = top - CAPITAL_H;
    let y1 = top - PANEL_T;
    let (qx, qz) = (hx + CAPITAL_OUT, hz + CAPITAL_OUT);
    let (dx, dz) = (hx + PANEL_OUT, hz + PANEL_OUT);
    let along = |p: &Vec3, n: &Vec3| (n.z * p.x - n.x * p.z) / scale.0;
    // a quad by its four corners (counter-clockwise seen from outside) and its normal
    let mut quad = |a: Vec3, b: Vec3, c: Vec3, d: Vec3, n: Vec3, under: bool| {
        let uv = |p: &Vec3| {
            if under { (p.x / scale.0, (p.y + 0.25 * (p.z - cz)) / scale.1) } else { (along(p, &n), p.y / scale.1) }
        };
        let i = f.vert(a, n, uv(&a));
        f.vert(b, n, uv(&b));
        f.vert(c, n, uv(&c));
        f.vert(d, n, uv(&d));
        f.idx.extend([i, i + 1, i + 2, i, i + 2, i + 3]);
    };
    let v = |x: f64, y: f64, z: f64| Vec3::new(cx + x, y, cz + z);
    // the flare: four sloping faces from the column's to the capital's top
    let slope = |out: f64| {
        let l = (out * out + (y1 - y0) * (y1 - y0)).sqrt();
        ((y1 - y0) / l, -out / l)
    };
    let (hn, vn) = slope(CAPITAL_OUT);
    // +x face
    quad(v(hx, y0, hz), v(hx, y0, -hz), v(qx, y1, -qz), v(qx, y1, qz), Vec3::new(hn, vn, 0.0), false);
    // -x face
    quad(v(-hx, y0, -hz), v(-hx, y0, hz), v(-qx, y1, qz), v(-qx, y1, -qz), Vec3::new(-hn, vn, 0.0), false);
    // +z face
    quad(v(-hx, y0, hz), v(hx, y0, hz), v(qx, y1, qz), v(-qx, y1, qz), Vec3::new(0.0, vn, hn), false);
    // -z face
    quad(v(hx, y0, -hz), v(-hx, y0, -hz), v(-qx, y1, -qz), v(qx, y1, -qz), Vec3::new(0.0, vn, -hn), false);
    // the drop panel: its four sides, set a hair into the slab, and its underside
    let yt = top + 0.002;
    quad(v(dx, y1, dz), v(dx, y1, -dz), v(dx, yt, -dz), v(dx, yt, dz), Vec3::new(1.0, 0.0, 0.0), false);
    quad(v(-dx, y1, -dz), v(-dx, y1, dz), v(-dx, yt, dz), v(-dx, yt, -dz), Vec3::new(-1.0, 0.0, 0.0), false);
    quad(v(-dx, y1, dz), v(dx, y1, dz), v(dx, yt, dz), v(-dx, yt, dz), Vec3::new(0.0, 0.0, 1.0), false);
    quad(v(dx, y1, -dz), v(-dx, y1, -dz), v(-dx, yt, -dz), v(dx, yt, -dz), Vec3::new(0.0, 0.0, -1.0), false);
    quad(v(-dx, y1, -dz), v(dx, y1, -dz), v(dx, y1, dz), v(-dx, y1, dz), Vec3::new(0.0, -1.0, 0.0), true);
}
