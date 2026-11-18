//! Height in the rooms, as drops seen into (`docs/plan/FACILITY.md` F7, the
//! developer's answer: walkable lower floors are F15). In some of the
//! Habitable Zone's bare halls a block of the floor is cut away: a pit two or
//! three tiles across, its sides concrete, its bottom a floor or standing
//! water three metres down, or the dark (rule 4 as eased: never the void, no
//! fog rises from it, nothing stands across it, nothing looks back out of
//! it: no catwalk crosses a dark one). Railed all round, to the
//! code (a top rail at 1.07 m, a mid rail, a toe board, posts at most 2.4 m
//! apart: the research), and the drop's own solid stops a body at its edge,
//! as the Building's voids do: nobody falls. Across some a catwalk of bar
//! grating, railed both sides.
//!
//! The floor stays one storey: a pit's tiles are wall tiles of the grid (so
//! nothing paths, places or camps on them) whose solid is a void
//! (`BOX_VOID`: it stops a body and no look). A catwalk's are floor tiles of
//! the grid, so whatever walks crosses it as you do (no refuge), its walkway
//! left clear between two strips of void; nothing is placed, camped or
//! hung on one (`Game::is_drop_tile`), nor on a pit's edge as on a wall's.
//! The walls are built as the floor was before the cut, so a column on a
//! drop's edge keeps its shape. Where they are is planned
//! before the floor is built, from a stream of its own (no draw of the level
//! moves); what is drawn of them is laid once the floor has settled
//! (`Game::lay_drops`), and what was hung over one is taken away.

use crate::base::rng::SplitMix64;
use crate::level::archgen::{BOX_FLOOR, BOX_VOID, Solids};
use crate::prelude::*;
use crate::visuals::lighting::RF_MOSS;
use crate::world::arch::grime_of;
use crate::world::building::along;
use crate::world::lining::Face;

/// A drop's tile, in `Game::drop_tiles`
pub const DROP_PIT: u8 = 1;
/// A catwalk's tile over a drop, in `Game::drop_tiles`
pub const DROP_CATWALK: u8 = 2;
/// How deep a drop's floor or water is (m)
pub const DROP_DEPTH: f64 = 3.2;
/// How far below the floor a dark drop's sides go (m), into black
pub const DARK_DEPTH: f64 = 12.0;
/// A catwalk's walkway (m): between its rails
pub const CATWALK_W: f64 = 1.5;
/// The railing: its top rail's height (42 in), and posts at most this far
/// apart (8 ft)
pub const RAIL_H: f64 = 1.07;
pub const POST_EVERY: f64 = 2.4;
/// The floor the drops are cut into: from this game level down
pub const DROP_FIRST_LEVEL: i32 = 5;
/// The drops' own stream
const DROP_SALT: u64 = 0x0D20_9F1A_77E5_3C41;

/// What is at a drop's bottom
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bottom {
    Floor,
    Water,
    Dark,
}

/// One drop: its tiles' block (inclusive), what is at its bottom, the
/// catwalk across it if it has one (along x or z, at that row or column),
/// and what is a pit hall's tank's own if it is one (`world::pits`)
#[derive(Clone, Debug)]
pub struct Drop {
    pub x0: i32,
    pub z0: i32,
    pub x1: i32,
    pub z1: i32,
    pub bottom: Bottom,
    pub catwalk: Option<(bool, i32)>,
    pub tank: Option<crate::world::pits::Tank>,
}

impl Drop {
    fn has(&self, x: i32, z: i32) -> bool {
        x >= self.x0 && x <= self.x1 && z >= self.z0 && z <= self.z1
    }
    fn catwalk_at(&self, x: i32, z: i32) -> bool {
        match self.catwalk {
            Some((true, row)) => z == row && self.has(x, z),
            Some((false, col)) => x == col && self.has(x, z),
            None => false,
        }
    }
}

impl Game {
    /// Plan the floor's drops (see the module): on the rooms' floors from
    /// `DROP_FIRST_LEVEL`, in a bare hall's open floor, the ring round it its
    /// floor or its columns, clear of where she comes in; at most one a hall,
    /// only where every open tile is still reached. Their tiles become wall
    /// tiles. A pit hall's tanks first (`world::pits`).
    pub fn plan_drops(&mut self) {
        self.drops.clear();
        self.drop_tiles = vec![0; self.tiles.len()];
        if self.level_num < DROP_FIRST_LEVEL || self.level_scp.is_some() || !self.ground.is_empty() {
            return;
        }
        self.plan_pits();
        let Some(l) = self.layout.as_ref() else { return };
        let (w, h) = (self.w, self.h);
        let zones: Vec<(usize, i32, i32, i32, i32)> = l
            .zones
            .iter()
            .enumerate()
            .filter(|(_, z)| z.arch == crate::world::halls::HALL)
            .map(|(i, z)| (i, z.x0, z.z0, z.x1, z.z1))
            .collect();
        let zone_of = l.zone_of.clone();
        let open = |g: &Game, x: i32, z: i32| x >= 0 && z >= 0 && x < w && z < h && g.tiles[(z * w + x) as usize] == 0;
        for (zi, zx0, zz0, zx1, zz1) in zones {
            let mut r = SplitMix64::new(self.current_seed, DROP_SALT ^ (zi as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
            if r.next() >= 0.45 {
                continue;
            }
            // the sizes it tries, larger first, and where
            let sizes: [(i32, i32); 4] = [(3, 3), (3, 2), (2, 3), (2, 2)];
            let mut found = None;
            'size: for (bw, bh) in sizes {
                let mut spots = vec![];
                for z in zz0..=zz1 - bh + 1 {
                    for x in zx0..=zx1 - bw + 1 {
                        spots.push((x, z));
                    }
                }
                // a shuffle of its own, so not always the zone's corner
                for i in (1..spots.len()).rev() {
                    let j = (r.next() * (i + 1) as f64) as usize;
                    spots.swap(i, j);
                }
                for (x, z) in spots {
                    if self.drop_fits(x, z, bw, bh, zi, &zone_of, &open) {
                        found = Some((x, z, bw, bh));
                        break 'size;
                    }
                }
            }
            let Some((x, z, bw, bh)) = found else { continue };
            let pick = r.next();
            let bottom = if pick < 0.5 {
                Bottom::Floor
            } else if pick < 0.8 {
                Bottom::Water
            } else {
                Bottom::Dark
            };
            // a catwalk along its long side's middle, half the time where it
            // is three tiles across
            let catwalk = if r.next() < 0.5 {
                if bw == 3 {
                    Some((false, x + 1))
                } else if bh == 3 {
                    Some((true, z + 1))
                } else {
                    None
                }
            } else {
                None
            };
            // (only where it lands on the floor at both ends, not on a
            // column; and never across the dark: nothing stands across it)
            let catwalk = catwalk.filter(|&(along_x, k)| {
                let ends = if along_x { [(x - 1, k), (x + bw, k)] } else { [(k, z - 1), (k, z + bh)] };
                bottom != Bottom::Dark && ends.iter().all(|&(ex, ez)| open(self, ex, ez))
            });
            self.cut_drop(Drop { x0: x, z0: z, x1: x + bw - 1, z1: z + bh - 1, bottom, catwalk, tank: None });
        }
    }

    /// Cut drop `d` into the floor: its pit's tiles walls (its catwalk's
    /// stay floor: what walks crosses it), kept only if every open tile is
    /// still reached (else the floor is put back). → it was cut
    pub(crate) fn cut_drop(&mut self, d: Drop) -> bool {
        let w = self.w;
        let saved: Vec<(usize, u8)> = (d.z0..=d.z1)
            .flat_map(|zz| (d.x0..=d.x1).map(move |xx| (zz, xx)))
            .filter(|&(zz, xx)| !d.catwalk_at(xx, zz))
            .map(|(zz, xx)| {
                let t = (zz * w + xx) as usize;
                (t, self.tiles[t])
            })
            .collect();
        for &(t, _) in &saved {
            self.tiles[t] = 1;
        }
        if !self.every_open_reached() {
            for (t, v) in saved {
                self.tiles[t] = v;
            }
            return false;
        }
        for zz in d.z0..=d.z1 {
            for xx in d.x0..=d.x1 {
                self.drop_tiles[(zz * w + xx) as usize] = if d.catwalk_at(xx, zz) { DROP_CATWALK } else { DROP_PIT };
            }
        }
        self.drops.push(d);
        true
    }

    /// A block of `bw × bh` tiles at (`x`, `z`) can be a drop: all open, the
    /// hall's (bare), its ring the hall's floor or its columns, and clear of
    /// where she comes in
    fn drop_fits(&self, x: i32, z: i32, bw: i32, bh: i32, zi: usize, zone_of: &[u8], open: &dyn Fn(&Game, i32, i32) -> bool) -> bool {
        let (w, h) = (self.w, self.h);
        // clear of the spawn (1, 1) by three tiles
        if x <= 4 && z <= 4 {
            return false;
        }
        for zz in z - 1..=z + bh {
            for xx in x - 1..=x + bw {
                if xx < 1 || zz < 1 || xx >= w - 1 || zz >= h - 1 {
                    return false;
                }
                let t = (zz * w + xx) as usize;
                if zone_of[t] as usize != zi || self.region_info[self.region_map[t] as usize].flags & RF_MOSS != 0 {
                    return false;
                }
                let inner = xx >= x && xx < x + bw && zz >= z && zz < z + bh;
                if inner {
                    if !open(self, xx, zz) {
                        return false;
                    }
                } else if !open(self, xx, zz) {
                    // a column: a wall tile with open floor all round it
                    let column = [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().all(|&(dx, dz)| {
                        open(self, xx + dx, zz + dz) || {
                            let (nx, nz) = (xx + dx, zz + dz);
                            nx >= x && nx < x + bw && nz >= z && nz < z + bh
                        }
                    });
                    if !column {
                        return false;
                    }
                }
            }
        }
        true
    }

    /// Every open tile is reached from her way in (1, 1)
    fn every_open_reached(&self) -> bool {
        let dist = self.bfs(1, 1);
        (0..self.tiles.len()).all(|t| self.tiles[t] != 0 || dist[t] >= 0)
    }

    /// The drop's boxes in place of what the walls would have made of its
    /// tiles: a void a pit tile, and a catwalk's two strips beside its walkway
    pub fn drop_solids(&self, s: Solids) -> Solids {
        if self.drops.is_empty() {
            return s;
        }
        let mut boxes: Vec<f32> = Vec::with_capacity(s.boxes.len());
        for b in 0..s.count {
            let bx = s.box_at(b);
            let owner = bx[6] as usize;
            if self.drop_tiles.get(owner).copied().unwrap_or(0) != 0 {
                continue;
            }
            boxes.extend_from_slice(bx);
        }
        let side = (1.0 - CATWALK_W / CELL) / 2.0;
        let mut push = |x0: f64, z0: f64, x1: f64, z1: f64, t: usize| {
            boxes.extend([x0 as f32, z0 as f32, x1 as f32, z1 as f32, -1.0e4, 1.0e4, t as f32, BOX_VOID as f32]);
        };
        for d in &self.drops {
            for z in d.z0..=d.z1 {
                for x in d.x0..=d.x1 {
                    let t = (z * self.w + x) as usize;
                    let (xf, zf) = (x as f64, z as f64);
                    if !d.catwalk_at(x, z) {
                        push(xf, zf, xf + 1.0, zf + 1.0, t);
                    } else if let Some((true, _)) = d.catwalk {
                        // the walkway along x: void either side of it along z
                        push(xf, zf, xf + 1.0, zf + side, t);
                        push(xf, zf + 1.0 - side, xf + 1.0, zf + 1.0, t);
                    } else {
                        push(xf, zf, xf + side, zf + 1.0, t);
                        push(xf + 1.0 - side, zf, xf + 1.0, zf + 1.0, t);
                    }
                }
            }
        }
        crate::level::archgen::make_solids(s.w, s.h, s.cell, boxes)
    }

    /// Tile (`x`, `z`) is a drop's: a pit's (a wall tile with no wall to
    /// hang a thing on) or its catwalk's (a floor tile nothing is put on)
    pub fn is_drop_tile(&self, x: i32, z: i32) -> bool {
        self.in_grid(x, z) && self.drop_tiles.get(self.t_idx(x, z)).is_some_and(|&d| d != 0)
    }

    /// The floor of a drop at world (`x`, `z`), if it is over one and off its
    /// catwalk's walkway: its bottom, or as deep as a dark one's sides go (the
    /// light map's floor, so its sides are lit as a place's are toward their
    /// own floor: `visuals::lighting`)
    pub fn drop_floor_at(&self, x: f64, z: f64) -> Option<f64> {
        for d in &self.drops {
            let (x0, z0, x1, z1) = self.drop_rect(d);
            if x <= x0 || x >= x1 || z <= z0 || z >= z1 {
                continue;
            }
            if let Some((along_x, m)) = self.catwalk_mid(d)
                && (if along_x { (z - m).abs() } else { (x - m).abs() }) < CATWALK_W / 2.0
            {
                return None;
            }
            return Some(match (&d.tank, d.bottom) {
                (Some(t), _) => -t.water,
                (_, Bottom::Dark) => -DARK_DEPTH,
                _ => -DROP_DEPTH,
            });
        }
        None
    }

    /// The floor's outer skirt (a plane a little under the floor, so its
    /// edge shows no void) with the drops left open: strips between them
    pub fn skirt_round_drops(&self, mat: Mat) -> Obj {
        let (ex, ez) = (self.w as f64 * CELL, self.h as f64 * CELL);
        let rects: Vec<(f64, f64, f64, f64)> = self.drops.iter().map(|d| self.drop_rect(d)).collect();
        let mut xs: Vec<f64> = vec![-ex, ex];
        for &(x0, _, x1, _) in &rects {
            xs.push(x0);
            xs.push(x1);
        }
        xs.sort_by(f64::total_cmp);
        xs.dedup();
        let up = Vec3::new(0.0, 1.0, 0.0);
        let y = -0.02;
        let mut f = Face::default();
        for xw in xs.windows(2) {
            let (a, b) = (xw[0], xw[1]);
            if b - a < 1e-6 {
                continue;
            }
            let mid = (a + b) / 2.0;
            // the drops this strip crosses, as z intervals, and what is between
            let mut cuts: Vec<(f64, f64)> = rects.iter().filter(|r| mid > r.0 && mid < r.2).map(|r| (r.1, r.3)).collect();
            cuts.sort_by(|p, q| p.0.total_cmp(&q.0));
            let mut z = -ez;
            for (c0, c1) in cuts.into_iter().chain(std::iter::once((ez, ez))) {
                if c0 > z {
                    let q = [Vec3::new(a, y, z), Vec3::new(b, y, z), Vec3::new(b, y, c0), Vec3::new(a, y, c0)];
                    f.quad(q, up, |_| (0.0, 0.0), None);
                }
                z = z.max(c1);
            }
        }
        f.mesh(&mat)
    }

    /// A wall's solid (not a drop's void, not a floor) stands at world
    /// (`x`, `z`), or within a post's half-width of it
    fn wall_at(&self, x: f64, z: f64) -> bool {
        let Some(s) = self.solids.as_ref() else { return false };
        let (tx, tz) = (self.tx(x), self.tz(z));
        if !self.in_grid(tx, tz) {
            return true;
        }
        let (u, v) = (self.to_u(x), self.to_v(z));
        let m = 0.03 / CELL;
        let t = self.t_idx(tx, tz);
        (s.start[t]..s.start[t + 1]).any(|k| {
            let b = s.box_at(s.items[k as usize] as usize);
            let kind = b[7] as i32;
            kind != BOX_VOID
                && kind != BOX_FLOOR
                && (b[0] as f64) < u + m
                && (b[2] as f64) > u - m
                && (b[1] as f64) < v + m
                && (b[3] as f64) > v - m
        })
    }

    /// The block of a drop in world metres (x0, z0, x1, z1)
    fn drop_rect(&self, d: &Drop) -> (f64, f64, f64, f64) {
        let (w, h) = (self.w as f64, self.h as f64);
        (
            (d.x0 as f64 - w / 2.0) * CELL,
            (d.z0 as f64 - h / 2.0) * CELL,
            (d.x1 as f64 + 1.0 - w / 2.0) * CELL,
            (d.z1 as f64 + 1.0 - h / 2.0) * CELL,
        )
    }

    /// The walkway of a drop's catwalk in world metres (its middle line's
    /// coordinate across it), if it has one
    fn catwalk_mid(&self, d: &Drop) -> Option<(bool, f64)> {
        let (w, h) = (self.w as f64, self.h as f64);
        d.catwalk.map(
            |(along_x, k)| if along_x { (true, (k as f64 + 0.5 - h / 2.0) * CELL) } else { (false, (k as f64 + 0.5 - w / 2.0) * CELL) },
        )
    }

    /// Take away what settling left over or against a drop: a thing hung on
    /// its edge, as on a wall's, or stood against its rail's line (from the
    /// world's child `start`; the floor's pickups and gameplay kept)
    pub fn clear_drops_of_props(&mut self, start: usize) {
        if self.drops.is_empty() {
            return;
        }
        let rects: Vec<(f64, f64, f64, f64)> = self.drops.iter().map(|d| self.drop_rect(d)).collect();
        let keep: Vec<Obj> = self.almonds.iter().chain(self.battery_items.iter()).map(|p| p.g.clone()).collect();
        let mut gone = vec![];
        for top in self.world.children().into_iter().skip(start) {
            if top.user().gameplay || keep.iter().any(|k| k.ptr_eq(&top)) {
                continue;
            }
            let mut bb = Box3::empty();
            bb.set_from_object(&top);
            if bb.is_empty() {
                continue;
            }
            let near = 0.1;
            if rects
                .iter()
                .any(|&(x0, z0, x1, z1)| bb.max.x > x0 - near && bb.min.x < x1 + near && bb.max.z > z0 - near && bb.min.z < z1 + near)
            {
                gone.push(top);
            }
        }
        for top in gone {
            self.world.remove(&top);
        }
    }

    /// What the floor's drops are drawn as (see the module): their sides
    /// and bottoms, their rails and their catwalks
    pub fn lay_drops(&mut self) {
        if self.drops.is_empty() {
            return;
        }
        let drops = self.drops.clone();
        let up = Vec3::new(0.0, 1.0, 0.0);
        let mut sides: Vec<(String, Face)> = vec![];
        let mut wet_sides: Vec<(String, Face)> = vec![];
        let mut dark_sides: Vec<(String, Face)> = vec![];
        let mut floors: Vec<(String, Face)> = vec![];
        let mut blacks = Face::default();
        let mut water: Vec<Mat4> = vec![];
        let mut tank_water: Vec<Mat4> = vec![];
        let mut rails: Vec<(bool, Mat4)> = vec![];
        let mut decks: Vec<Mat4> = vec![];
        let mut stringers: Vec<Mat4> = vec![];
        let into = |v: &mut Vec<(String, Face)>, key: &str| -> usize {
            match v.iter().position(|(k, _)| k == key) {
                Some(i) => i,
                None => {
                    v.push((key.to_string(), Face::default()));
                    v.len() - 1
                }
            }
        };
        for d in &drops {
            let (x0, z0, x1, z1) = self.drop_rect(d);
            let t = (d.z0 * self.w + d.x0) as usize;
            let g = grime_of(&self.region_info[self.region_map[t] as usize]);
            let key = format!("formwork:{g}");
            let depth = if d.bottom == Bottom::Dark { DARK_DEPTH } else { DROP_DEPTH };
            // its four sides, facing in, from the floor down; a dark one's
            // going black row by row
            let walls = [
                ([Vec3::new(x0, 0.0, z0), Vec3::new(x1, 0.0, z0)], Vec3::new(0.0, 0.0, 1.0)),
                ([Vec3::new(x1, 0.0, z1), Vec3::new(x0, 0.0, z1)], Vec3::new(0.0, 0.0, -1.0)),
                ([Vec3::new(x0, 0.0, z1), Vec3::new(x0, 0.0, z0)], Vec3::new(1.0, 0.0, 0.0)),
                ([Vec3::new(x1, 0.0, z0), Vec3::new(x1, 0.0, z1)], Vec3::new(-1.0, 0.0, 0.0)),
            ];
            let rows: Vec<f64> =
                if d.bottom == Bottom::Dark { vec![0.0, -1.0, -2.2, -3.6, -5.4, -8.0, -DARK_DEPTH] } else { vec![0.0, -depth] };
            let shade = |y: f64| if d.bottom == Bottom::Dark { (y / 2.4).dexp().clamp(0.0, 1.0) } else { 1.0 };
            // a pit hall's tank: its walls wet above its water, its columns
            // rising out of it, its water (`world::pits`)
            if let Some(tk) = &d.tank {
                let (i, j) = (into(&mut sides, &key), into(&mut wet_sides, &key));
                for ([a, b], n) in walls {
                    crate::world::pits::tank_face(&mut sides[i].1, &mut wet_sides[j].1, a, b, n, 0.0, tk);
                }
                for &(cx, cz) in &tk.columns {
                    let ceil = self.tile_ceil.get(self.t_idx(self.tx(cx), self.tz(cz))).map_or(WALL_H, |&c| c as f64);
                    crate::world::pits::tank_column(&mut sides[i].1, &mut wet_sides[j].1, (cx, cz), ceil, tk);
                }
                tank_water.push(Mat4::compose(
                    &Vec3::new((x0 + x1) / 2.0, -tk.water, (z0 + z1) / 2.0),
                    &Quat::IDENTITY,
                    &Vec3::new(x1 - x0, 1.0, z1 - z0),
                ));
            }
            for ([a, b], n) in walls.into_iter().filter(|_| d.tank.is_none()) {
                let along_uv = move |p: &Vec3| ((n.z * p.x - n.x * p.z) / 4.0, p.y / 4.0);
                for r in rows.windows(2) {
                    let (ya, yb) = (r[0], r[1]);
                    let q = [Vec3::new(a.x, ya, a.z), Vec3::new(b.x, ya, b.z), Vec3::new(b.x, yb, b.z), Vec3::new(a.x, yb, a.z)];
                    if d.bottom == Bottom::Dark {
                        let i = into(&mut dark_sides, &key);
                        dark_sides[i].1.quad(q, n, along_uv, Some([shade(ya), shade(ya), shade(yb), shade(yb)]));
                    } else {
                        let i = into(&mut sides, &key);
                        sides[i].1.quad(q, n, along_uv, None);
                    }
                }
            }
            // its bottom
            let bot = [Vec3::new(x0, -depth, z0), Vec3::new(x1, -depth, z0), Vec3::new(x1, -depth, z1), Vec3::new(x0, -depth, z1)];
            match d.bottom {
                _ if d.tank.is_some() => {}
                Bottom::Floor => {
                    let fkey = format!("floor:concrete:{g}");
                    let i = into(&mut floors, &fkey);
                    floors[i].1.quad(bot, up, |p: &Vec3| (p.x / CELL, p.z / CELL), None);
                }
                Bottom::Water => {
                    let i = into(&mut floors, &format!("floor:concrete:{g}"));
                    floors[i].1.quad(bot, up, |p: &Vec3| (p.x / CELL, p.z / CELL), None);
                    let mut m = Mat4::IDENTITY;
                    m.make_translation((x0 + x1) / 2.0, -depth + 0.55, (z0 + z1) / 2.0);
                    let mut s = Mat4::IDENTITY;
                    s.make_scale(x1 - x0, 1.0, z1 - z0);
                    m.multiply(&s);
                    water.push(m);
                }
                Bottom::Dark => blacks.quad(bot, up, |_| (0.0, 0.0), None),
            }
            // the rails: round its edge on the floor's side (where a catwalk
            // meets it, open for its walkway), and along its catwalk
            let off = 0.04;
            let half = CATWALK_W / 2.0;
            let cw = self.catwalk_mid(d);
            let edge = |a: Vec3, b: Vec3, rails: &mut Vec<(bool, Mat4)>| {
                let len = (b.x - a.x).dhypot(b.z - a.z);
                if len < 0.2 {
                    return;
                }
                rails.push((true, along(a, b, 1.0, 0.05, 0.05)));
                rails.push((false, along(a, b, 0.5, 0.04, 0.04)));
                rails.push((true, along(a, b, 0.0, 0.12, 0.015)));
                let k = (len / POST_EVERY).ceil().max(1.0) as usize;
                for i in 0..=k {
                    let f = i as f64 / k as f64;
                    let p = Vec3::new(a.x + (b.x - a.x) * f, 0.0, a.z + (b.z - a.z) * f);
                    rails.push((false, Mat4::compose(&Vec3::new(p.x, 0.51, p.z), &Quat::IDENTITY, &Vec3::new(0.045, 1.03, 0.045))));
                }
            };
            // the edges along x (z0 and z1) and along z (x0 and x1), each
            // split where a catwalk meets it; an end whose corner is in a
            // wall (a pier filling the tile across the corner) stops short
            // of it, at the drop's line (the rails ran into one, the visual
            // playtest)
            let (ex0, ez0, ex1, ez1) = (x0 - off, z0 - off, x1 + off, z1 + off);
            let short = 0.005;
            for (zz, mouth) in [(ez0, cw.filter(|c| !c.0)), (ez1, cw.filter(|c| !c.0))] {
                let a = if self.wall_at(ex0, zz) { x0 + short } else { ex0 };
                let b = if self.wall_at(ex1, zz) { x1 - short } else { ex1 };
                match mouth {
                    Some((_, m)) => {
                        edge(Vec3::new(a, 0.0, zz), Vec3::new(m - half, 0.0, zz), &mut rails);
                        edge(Vec3::new(m + half, 0.0, zz), Vec3::new(b, 0.0, zz), &mut rails);
                    }
                    None => edge(Vec3::new(a, 0.0, zz), Vec3::new(b, 0.0, zz), &mut rails),
                }
            }
            for (xx, mouth) in [(ex0, cw.filter(|c| c.0)), (ex1, cw.filter(|c| c.0))] {
                let a = if self.wall_at(xx, ez0) { z0 + short } else { ez0 };
                let b = if self.wall_at(xx, ez1) { z1 - short } else { ez1 };
                match mouth {
                    Some((_, m)) => {
                        edge(Vec3::new(xx, 0.0, a), Vec3::new(xx, 0.0, m - half), &mut rails);
                        edge(Vec3::new(xx, 0.0, m + half), Vec3::new(xx, 0.0, b), &mut rails);
                    }
                    None => edge(Vec3::new(xx, 0.0, a), Vec3::new(xx, 0.0, b), &mut rails),
                }
            }
            // the catwalk: its deck of bar grating from rim to rim, two
            // stringers under its edges, a rail along each side
            if let Some((along_x, m)) = cw {
                let (a, b) =
                    if along_x { (Vec3::new(x0, 0.0, m), Vec3::new(x1, 0.0, m)) } else { (Vec3::new(m, 0.0, z0), Vec3::new(m, 0.0, z1)) };
                decks.push(along(a, b, -0.04, 0.04, CATWALK_W));
                for s in [-1.0, 1.0] {
                    let o = s * (half - 0.05);
                    let (sa, sb) = if along_x {
                        (Vec3::new(a.x, 0.0, a.z + o), Vec3::new(b.x, 0.0, b.z + o))
                    } else {
                        (Vec3::new(a.x + o, 0.0, a.z), Vec3::new(b.x + o, 0.0, b.z))
                    };
                    // (a centimetre under the deck: their tops and its
                    // underside one plane fought through the grating)
                    stringers.push(along(sa, sb, -0.3, 0.25, 0.08));
                    let ro = s * (half - 0.03);
                    let (ra, rb) = if along_x {
                        (Vec3::new(a.x, 0.0, a.z + ro), Vec3::new(b.x, 0.0, b.z + ro))
                    } else {
                        (Vec3::new(a.x + ro, 0.0, a.z), Vec3::new(b.x + ro, 0.0, b.z))
                    };
                    edge(ra, rb, &mut rails);
                }
            }
        }
        // the meshes
        for (key, f) in sides {
            let mat = self.arch_mat(&key);
            let m = f.mesh(&mat);
            m.user().arch = true;
            self.world.add(&m);
        }
        for (key, f) in wet_sides {
            let mat = crate::world::pits::wet_mat(&self.arch_mat(&key));
            let m = f.mesh(&mat);
            m.user().arch = true;
            self.world.add(&m);
        }
        for (key, f) in dark_sides {
            let mat = self.arch_mat(&key).duplicate();
            mat.m().vertex_colors = true;
            let m = f.mesh(&mat);
            m.user().arch = true;
            self.world.add(&m);
        }
        for (key, f) in floors {
            let mat = self.arch_mat(&key);
            let m = f.mesh(&mat);
            m.user().arch = true;
            self.world.add(&m);
        }
        if !blacks.idx.is_empty() {
            let m = blacks.mesh(&basic_mat!({ color: 0x000000 }));
            m.user().arch = true;
            self.world.add(&m);
        }
        let unit = self.unit_box_geo.clone();
        if !water.is_empty() {
            // standing water, dark, holding the lamps' gloss (its colour a
            // murky water's albedo, about 0.01: the materials test's floor)
            let mat = phong!({ color: 0x1c201c, specular: 0x6a6a68, shininess: 90 });
            let geo = plane_geo!(1.0, 1.0);
            geo.rotate_x(-std::f64::consts::FRAC_PI_2);
            self.add_instanced(&geo, &mat, &water, &|m: &Mat4| *m);
        }
        if !tank_water.is_empty() {
            let geo = plane_geo!(1.0, 1.0);
            geo.rotate_x(-std::f64::consts::FRAC_PI_2);
            self.add_instanced(&geo, &crate::world::pits::water_mat(), &tank_water, &|m: &Mat4| *m);
        }
        // the Works' rail: yellow top rail and toe board, steel posts and mid rail
        let yellow = lambert!({ color: 0x8c7a34 });
        let steel = lambert!({ color: 0x484b49 });
        crate::visuals::prop_look::settle_material(&yellow);
        crate::visuals::prop_look::settle_material(&steel);
        let ys: Vec<Mat4> = rails.iter().filter(|(y, _)| *y).map(|(_, m)| *m).collect();
        let ss: Vec<Mat4> = rails.iter().filter(|(y, _)| !*y).map(|(_, m)| *m).collect();
        self.add_instanced(&unit, &yellow, &ys, &|m: &Mat4| *m);
        self.add_instanced(&unit, &steel, &ss, &|m: &Mat4| *m);
        if !decks.is_empty() {
            // (a low cut: its mips' alpha is the bars' share of the floor,
            // about a quarter, and a grating seen far off is solid)
            let mat = lambert!({ map: &grating_tex(), alphaTest: 0.12, side: DoubleSide });
            crate::visuals::prop_look::settle_material(&mat);
            self.add_instanced(&unit, &mat, &decks, &|m: &Mat4| *m);
            self.add_instanced(&unit, &steel, &stringers, &|m: &Mat4| *m);
        }
    }
}

/// Bar grating seen from above (19-W-4: bearing bars 19 mm apart, cross
/// rods 100 mm), a tile of it 0.4 m square, the gaps clear. Mipmapped, the
/// gaps the bars' colour under no alpha: from afar it averages to the dull
/// grey a grating is at a glance (unfiltered, its bars finer than a pixel
/// shimmered: the visual playtest's flicker, the blind review's aliasing)
fn grating_tex() -> Tex {
    thread_local! {
        static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() };
    }
    T.with(|c| {
        c.get_or_init(|| {
            const N: usize = 256;
            let mut px = vec![0u8; N * N * 4];
            for j in 0..N {
                for i in 0..N {
                    // 0.4 m: 21 bearing bars along, 4 cross rods across
                    let bar = (i as f64 * 21.0 / N as f64).fract() < 0.2;
                    let rod = (j as f64 * 4.0 / N as f64).fract() < 0.06;
                    let o = (j * N + i) * 4;
                    // galvanised steel gone dull, the rods a shade darker
                    let v = if rod { 70 } else { 80 };
                    px[o..o + 4].copy_from_slice(&[v, v, v - 3, if bar || rod { 255 } else { 0 }]);
                }
            }
            let t = Tex::data_rgba(px, N as u32, N as u32);
            {
                let mut tt = t.t();
                tt.wrap_s = Wrap::Repeat;
                tt.wrap_t = Wrap::Repeat;
                tt.repeat = Vec2::new(1.0 / 0.4, 1.0 / 0.4);
                tt.generate_mipmaps = true;
                tt.min_filter = Filter::LinearMipmapLinear;
                tt.mag_filter = Filter::Linear;
                tt.anisotropy = 4;
            }
            t
        })
        .clone()
    })
}
