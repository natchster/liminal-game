//! The level world: the maze and architecture, the world build, level decay,
//! the architecture meshes, teardown, prop draw distance, placement, settling
//! and prop themes.
//!
//! This file has the grid queries, the maze, the archgen layout, the zone /
//! ceiling / nav queries, the small audio-debug helpers, level decay and the
//! regions. [`arch`]: the architecture meshes, teardown and the prop draw
//! distance; [`settle`]: final placement and `settle_props`; [`placement`]:
//! the placement checks; [`build`]: the prop themes, `themed` and
//! `build_world`; [`purpose`]: what each zone is for; [`instancing`]:
//! static-prop instancing; [`tables`]: the build tables; [`zones`]: zone
//! record access.
//! CONTRACT: these signatures are called from other modules; keep them.
pub mod arch;
pub mod assembly;
pub mod build;
pub mod building;
pub mod building_dress;
pub mod instancing;
pub mod placement;
pub mod purpose;
pub mod settle;
pub mod tables;
pub mod zones;

use crate::base::numeric::hypot2;
use crate::base::records::*;
use crate::level::archgen;
use crate::prelude::*;
use crate::visuals::lighting::{RF_DARK, RF_MOSS};
use crate::visuals::textures::{PAL_MONO_YELLOW, WALL_PALETTES};

pub const DECAY_BASE: f64 = 0.20;
pub const DECAY_STEP: f64 = 0.10;
pub const DECAY_MAX: f64 = 0.92;
/// How brightly a region reads, by grime
pub const GRIME_LIGHT: [f64; 4] = [1.0, 0.78, 0.5, 0.3];
pub const RATE_DELTA_WINDOW: usize = 30;
pub const BLOWOUT_MILESTONE_AT: f64 = 25.0;
pub const RELIGHT_MILESTONE_AT: f64 = 15.0;
const N4: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

/// [`choice`] that tolerates an empty list: it still makes its one draw, then
/// returns None.
pub(crate) fn choice_opt<T: Clone>(a: &[T]) -> Option<T> {
    let i = (rnd() * a.len() as f64).floor() as usize;
    a.get(i).cloned()
}

/// The regions of a level (see `Game::region_units`)
struct RegionUnits {
    /// region per tile
    tile: Vec<usize>,
    /// surface per region
    surface: Vec<usize>,
    /// palette key per surface (None: a palette of its own)
    keys: Vec<Option<&'static str>>,
}

/// The pool room `build_world` reserved (`place_props` takes it)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PoolPlan {
    pub x: i32,
    pub z: i32,
    pub pw: i32,
    pub ph: i32,
}

impl Game {
    /// Start-up work of the world module (shared materials / caches)
    pub fn init_world(&mut self) {
        // `wall_mats` / `ceil_mats` / `floor_mats`: nothing else builds them
        // yet, and `build_floors` / `build_ceilings` need them
        self.ensure_surface_mats();
        self.placer_index_refresh();
    }

    // ------------------------------------------------------------ maze
    /// (the pre-archgen maze; kept for tools)
    pub fn gen_maze(&mut self, size: i32) {
        let n = if size % 2 != 0 { size } else { size + 1 };
        self.w = n;
        self.h = n;
        self.tiles = vec![1; (n * n) as usize];
        let (w, h) = (self.w, self.h);
        let mut stack: Vec<(i32, i32)> = vec![(1, 1)];
        let i11 = self.t_idx(1, 1);
        self.tiles[i11] = 0;
        let dirs = [(2, 0), (-2, 0), (0, 2), (0, -2)];
        while let Some(&(cx, cz)) = stack.last() {
            let opts: Vec<(i32, i32, i32, i32)> = dirs
                .iter()
                .map(|&(dx, dz)| (cx + dx, cz + dz, dx, dz))
                .filter(|&(nx, nz, _, _)| nx > 0 && nz > 0 && nx < w - 1 && nz < h - 1 && self.tiles[self.t_idx(nx, nz)] == 1)
                .collect();
            if opts.is_empty() {
                stack.pop();
                continue;
            }
            let (nx, nz, dx, dz) = choice(&opts);
            let a = self.t_idx(cx + dx / 2, cz + dz / 2);
            self.tiles[a] = 0;
            let b = self.t_idx(nx, nz);
            self.tiles[b] = 0;
            stack.push((nx, nz));
        }
        // knock down walls -> loops and open rooms
        for z in 1..h - 1 {
            for x in 1..w - 1 {
                if self.tiles[self.t_idx(x, z)] == 0 {
                    continue;
                }
                let h_open = !self.is_wall_tile(x - 1, z) && !self.is_wall_tile(x + 1, z);
                let v_open = !self.is_wall_tile(x, z - 1) && !self.is_wall_tile(x, z + 1);
                if (h_open != v_open) && rnd() < 0.30 {
                    let i = self.t_idx(x, z);
                    self.tiles[i] = 0;
                }
            }
        }
        // widen some corridors
        for z in 2..h - 2 {
            for x in 2..w - 2 {
                if self.tiles[self.t_idx(x, z)] != 0 {
                    continue;
                }
                let nb: Vec<bool> = [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().map(|&(dx, dz)| !self.is_wall_tile(x + dx, z + dz)).collect();
                let open_count = nb.iter().filter(|&&b| b).count();
                let horiz = nb[0] && nb[1] && !nb[2] && !nb[3];
                let vert = nb[2] && nb[3] && !nb[0] && !nb[1];
                if open_count != 2 || (!horiz && !vert) {
                    continue;
                }
                if rnd() < 0.22 {
                    let side = if rnd() < 0.5 { 1 } else { -1 };
                    let (ox, oz) = if horiz { (0, side) } else { (side, 0) };
                    if self.in_grid(x + ox, z + oz) && x + ox > 0 && z + oz > 0 && x + ox < w - 1 && z + oz < h - 1 {
                        let i = self.t_idx(x + ox, z + oz);
                        self.tiles[i] = 0;
                    }
                }
            }
        }
        // a few large plazas
        let plaza_count = 2.max((w * h) / 220);
        for _ in 0..plaza_count {
            let ww = irand(3, 5) as i32;
            let hh = irand(3, 5) as i32;
            let px = irand(3, (w - 3 - ww) as i64) as i32;
            let pz = irand(3, (h - 3 - hh) as i64) as i32;
            let mut touches = false;
            let mut zz = -1;
            while zz <= hh && !touches {
                let mut xx = -1;
                while xx <= ww && !touches {
                    let (gx, gz) = (px + xx, pz + zz);
                    if self.in_grid(gx, gz) && self.tiles[self.t_idx(gx, gz)] == 0 {
                        touches = true;
                    }
                    xx += 1;
                }
                zz += 1;
            }
            if !touches {
                continue;
            }
            for zz in 0..hh {
                for xx in 0..ww {
                    let (gx, gz) = (px + xx, pz + zz);
                    if gx > 0 && gz > 0 && gx < w - 1 && gz < h - 1 {
                        let i = self.t_idx(gx, gz);
                        self.tiles[i] = 0;
                    }
                }
            }
        }
        // lone pillars inside large open areas
        for z in 2..h - 2 {
            for x in 2..w - 2 {
                if self.tiles[self.t_idx(x, z)] == 1 {
                    continue;
                }
                let mut open = true;
                'o: for dz in -1..=1 {
                    for dx in -1..=1 {
                        if (dx != 0 || dz != 0) && self.is_wall_tile(x + dx, z + dz) {
                            open = false;
                            break 'o;
                        }
                    }
                }
                if open && rnd() < 0.05 {
                    let i = self.t_idx(x, z);
                    self.tiles[i] = 1;
                }
            }
        }
    }

    /// Tile distances from (sx, sz), -1 unreachable
    pub fn bfs(&self, sx: i32, sz: i32) -> Vec<i32> {
        let (w, h) = (self.w, self.h);
        let mut dist = vec![-1i32; (w * h).max(0) as usize];
        let (mut sx, mut sz) = (sx, sz);
        if self.is_wall_tile(sx, sz) {
            // the space beside a thin partition belongs to a wall tile; path
            // from the nearest open neighbour instead of giving up
            let mut found = false;
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (-1, 1), (1, -1), (-1, -1)] {
                if !self.is_wall_tile(sx + dx, sz + dz) {
                    sx += dx;
                    sz += dz;
                    found = true;
                    break;
                }
            }
            if !found {
                return dist;
            }
        }
        let mut q: Vec<usize> = vec![self.t_idx(sx, sz)];
        dist[q[0]] = 0;
        let mut qi = 0;
        while qi < q.len() {
            let cur = q[qi];
            let (cx, cz) = ((cur as i32) % w, (cur as i32) / w);
            let d = dist[cur] + 1;
            for (dx, dz) in N4 {
                let (nx, nz) = (cx + dx, cz + dz);
                if self.in_grid(nx, nz) {
                    let ni = self.t_idx(nx, nz);
                    if self.tiles[ni] == 0 && dist[ni] == -1 {
                        dist[ni] = d;
                        q.push(ni);
                    }
                }
            }
            qi += 1;
        }
        dist
    }

    /// Sight line at 1.5 m (world coords)
    pub fn los(&self, ax: f64, az: f64, bx: f64, bz: f64) -> bool {
        if let Some(s) = &self.solids {
            if !self.ground.is_empty() {
                // each end at 1.5 m over its own floor
                let (ya, yb) = (self.floor_y_at(ax, az) + 1.5, self.floor_y_at(bx, bz) + 1.5);
                return archgen::segment_clear_3d(s, self.to_u(ax), self.to_v(az), ya, self.to_u(bx), self.to_v(bz), yb);
            }
            return archgen::segment_clear(s, self.to_u(ax), self.to_v(az), self.to_u(bx), self.to_v(bz), 1.5);
        }
        let (dx, dz) = (bx - ax, bz - az);
        let d = hypot2(dx, dz);
        let steps = (d / 0.35).ceil();
        let mut i = 1.0;
        while i < steps {
            let t = i / steps;
            if self.is_wall_tile(self.tx(ax + dx * t), self.tz(az + dz * t)) {
                return false;
            }
            i += 1.0;
        }
        true
    }

    /// (world coords)
    pub fn circle_hits_wall(&self, px: f64, pz: f64, r: f64) -> bool {
        let (x0, x1, z0, z1) = (self.tx(px - r), self.tx(px + r), self.tz(pz - r), self.tz(pz + r));
        let hit_tile = |x: i32, z: i32| {
            let cx = clamp(px, self.wx(x) - CELL / 2.0, self.wx(x) + CELL / 2.0);
            let cz = clamp(pz, self.wz(z) - CELL / 2.0, self.wz(z) + CELL / 2.0);
            (px - cx).powi(2) + (pz - cz).powi(2) < r * r
        };
        if let Some(s) = &self.solids {
            // (what is over a body's head counts from the floor it stands on)
            if archgen::circle_hits(s, self.to_u(px), self.to_v(pz), r / CELL, self.floor_y_at(px, pz) + 1.8) {
                return true;
            }
            // a cell's hermetic door, shut
            if !self.cell_doors.is_empty() && self.hits_cell_door(px, pz, r) {
                return true;
            }
            if self.barricade_tiles.is_empty() {
                return false;
            }
            for z in z0..=z1 {
                for x in x0..=x1 {
                    if !self.in_grid(x, z) || !self.barricade_tiles.contains(&self.t_idx(x, z)) {
                        continue;
                    }
                    if hit_tile(x, z) {
                        return true;
                    }
                }
            }
            return false;
        }
        for z in z0..=z1 {
            for x in x0..=x1 {
                // treat barricade tiles as walls for collision purposes
                let blocked = self.is_wall_tile(x, z) || (self.in_grid(x, z) && self.barricade_tiles.contains(&self.t_idx(x, z)));
                if !blocked {
                    continue;
                }
                if hit_tile(x, z) {
                    return true;
                }
            }
        }
        false
    }

    /// (tile coords, blocked tile indices)
    pub fn alt_path_exists(&self, sx: i32, sz: i32, ex: i32, ez: i32, extra_blocked: &[usize]) -> bool {
        if self.is_wall_tile(sx, sz) || self.is_wall_tile(ex, ez) {
            return false;
        }
        let blocked: std::collections::HashSet<usize> = extra_blocked.iter().copied().collect();
        let mut visited = vec![0u8; (self.w * self.h) as usize];
        let mut q: Vec<(i32, i32)> = vec![(sx, sz)];
        visited[self.t_idx(sx, sz)] = 1;
        let mut qi = 0;
        while qi < q.len() {
            let (cx, cz) = q[qi];
            if cx == ex && cz == ez {
                return true;
            }
            for (dx, dz) in N4 {
                let (nx, nz) = (cx + dx, cz + dz);
                if !self.in_grid(nx, nz) {
                    continue;
                }
                let idx = self.t_idx(nx, nz);
                if visited[idx] != 0 || self.is_wall_tile(nx, nz) || blocked.contains(&idx) {
                    continue;
                }
                visited[idx] = 1;
                q.push((nx, nz));
            }
            qi += 1;
        }
        false
    }

    // ------------------------------------------------------------ architecture (archgen)
    pub fn gen_layout_level(&mut self, n: i32, size: i32) {
        let scp = self.level_scp.clone();
        let layout = archgen::generate_layout(&archgen::LayoutOpts { size, level: n, cell: CELL, scp: scp.as_deref() }, &mut || rnd());
        self.w = layout.w;
        self.h = layout.h;
        self.tiles = layout.tiles.clone();
        self.tile_ceil = archgen::ceiling_heights(&layout);
        self.ground = layout.ground.clone();
        self.building = None;
        self.layout = Some(layout);
        self.has_layout = true;
        // provisional solids so anything placed during the prop pass collides
        // with the real partitions; rebuilt with anchors once props are down
        self.solids = self.build_solids_now(None);
    }
    /// `build_solids` with `anchors` and `tile_ceil`, over the live `g.tiles`
    /// rather than the layout's copy
    pub fn build_solids_now(&self, anchors: Option<&std::collections::HashSet<usize>>) -> Option<archgen::Solids> {
        let l = self.layout.as_ref()?;
        // the Building's floors: masses, voids and the slabs under the floors
        if !self.ground.is_empty() {
            let extra = match &self.building {
                Some(p) => crate::level::building::cell_front_boxes(p, &self.ground, self.w, CELL),
                None => vec![],
            };
            return Some(archgen::build_ground_solids_with(self.w, self.h, CELL, &self.ground, &extra));
        }
        let view = archgen::LayoutView {
            w: self.w,
            h: self.h,
            cell: l.cell,
            tiles: &self.tiles,
            zone_of: &l.zone_of,
            tile_flags: Some(&l.tile_flags),
            zones: &l.zones,
        };
        Some(archgen::build_solids_view(&view, &archgen::SolidsOpts { anchors, ceil: Some(&self.tile_ceil), cell: CELL }))
    }

    // ------------------------------------------------------------ zone / ceiling / nav
    /// Index into the layout's zones
    pub fn zone_at_tile(&self, x: i32, z: i32) -> Option<usize> {
        let l = self.layout.as_ref()?;
        Some(l.zone_of[self.t_idx(x.clamp(0, (self.w - 1).max(0)), z.clamp(0, (self.h - 1).max(0)))] as usize)
    }
    /// `zone_at_tile(x, z)` as the zone itself
    pub fn zone_ref(&self, x: i32, z: i32) -> Option<&archgen::Zone> {
        let i = self.zone_at_tile(x, z)?;
        self.layout.as_ref().map(|l| &l.zones[i])
    }
    pub fn zone_at_world(&self, px: f64, pz: f64) -> Option<usize> {
        let (x, z) = (self.tx(px), self.tz(pz));
        self.zone_at_tile(x, z)
    }
    pub fn ceil_at_world(&self, px: f64, pz: f64) -> f64 {
        if self.tile_ceil.is_empty() {
            return WALL_H;
        }
        let x = self.tx(px).clamp(0, (self.w - 1).max(0));
        let z = self.tz(pz).clamp(0, (self.h - 1).max(0));
        self.tile_ceil.get(self.t_idx(x, z)).map(|&c| c as f64).unwrap_or(WALL_H)
    }
    /// Nearest open tile
    pub fn nav_tile_at(&self, px: f64, pz: f64) -> (i32, i32) {
        let (x, z) = (self.tx(px), self.tz(pz));
        if !self.is_wall_tile(x, z) {
            return (x, z);
        }
        let mut best = None;
        let mut bd = 1e9;
        for dz in -1..=1 {
            for dx in -1..=1 {
                if self.is_wall_tile(x + dx, z + dz) {
                    continue;
                }
                let d = (self.wx(x + dx) - px).powi(2) + (self.wz(z + dz) - pz).powi(2);
                if d < bd {
                    bd = d;
                    best = Some((x + dx, z + dz));
                }
            }
        }
        best.unwrap_or((x, z))
    }
    /// Slide along walls, per axis
    pub fn collide_move(&self, p: &mut Vec3, dx: f64, dz: f64, r: f64) {
        if dx != 0.0 && !dx.is_nan() && !self.circle_hits_wall(p.x + dx, p.z, r) {
            p.x += dx;
        }
        if dz != 0.0 && !dz.is_nan() && !self.circle_hits_wall(p.x, p.z + dz, r) {
            p.z += dz;
        }
    }
}

impl Game {
    // ------------------------------------------------------------ audio debug helpers
    /// A random world angle, biased away from camera-forward
    pub fn biased_spawn_angle(&self) -> f64 {
        let mut ang = rnd() * PI * 2.0;
        let yaw = if self.player.yaw.is_nan() { 0.0 } else { self.player.yaw };
        let (dx, dz) = (ang.cos(), ang.sin());
        let (fx, fz) = (-yaw.sin(), -yaw.cos());
        let dot = dx * fx + dz * fz;
        // dot > 0.5 → within ~60° of forward; reject 60% of the time by flipping
        if dot > 0.5 && rnd() < 0.6 {
            ang += PI;
        }
        ang
    }
    /// Stereo pan (-0.85..0.85) of a world position
    pub fn pan_for_world_pos(&self, wx: f64, wz: f64) -> f64 {
        let (dx, dz) = (wx - self.player.pos.x, wz - self.player.pos.z);
        let d0 = (dx * dx + dz * dz).sqrt();
        let d = if d0 == 0.0 || d0.is_nan() { 1.0 } else { d0 };
        let yaw = if self.player.yaw.is_nan() { 0.0 } else { self.player.yaw };
        let (rx, rz) = (yaw.cos(), -yaw.sin());
        let pan = (dx * rx + dz * rz) / d;
        clamp(pan, -1.0, 1.0) * 0.85
    }
    pub fn log_ambient(&mut self, tag: &str, world_x: Option<f64>, world_z: Option<f64>, pan: Option<f64>) {
        let mut entry = AmbientLogEntry { tag: tag.to_string(), at: self.clock_ms() / 1000.0, ..Default::default() };
        if let (Some(x), Some(z)) = (world_x, world_z) {
            let (dx, dz) = (x - self.player.pos.x, z - self.player.pos.z);
            entry.dist = Some((dx * dx + dz * dz).sqrt());
            // compass bearing — 0 rad = -Z is the player's "north"
            let ang_deg = (dx.atan2(-dz) * 180.0 / PI + 360.0) % 360.0;
            const DIRS: [&str; 8] = ["N", "NE", "E", "SE", "S", "SW", "W", "NW"];
            entry.bearing = Some(DIRS[(round_half_up(ang_deg / 45.0) as i64).rem_euclid(8) as usize].to_string());
        }
        if pan.is_some() {
            entry.pan = pan;
        }
        self.ambient_log.push(entry);
        if self.ambient_log.len() > 3 {
            self.ambient_log.remove(0);
        }
        // tally keyed by tag → weather → count
        let wk = {
            let k = self.weather_key();
            if k.is_empty() {
                "unknown"
            } else {
                k
            }
        };
        *self.ambient_tally.entry(tag.to_string()).or_default().entry(wk.to_string()).or_insert(0.0) += 1.0;
    }
    /// Pushes a rate delta into the window; returns the running average
    pub fn push_rate_delta(&mut self, delta: f64) -> Option<f64> {
        self.rate_delta_buf.push(delta);
        if self.rate_delta_buf.len() > RATE_DELTA_WINDOW {
            self.rate_delta_buf.remove(0);
        }
        if self.rate_delta_buf.is_empty() {
            return None;
        }
        Some(self.rate_delta_buf.iter().fold(0.0, |a, b| a + b) / self.rate_delta_buf.len() as f64)
    }
    pub fn current_rate_avg(&self) -> Option<f64> {
        if self.rate_delta_buf.is_empty() {
            return None;
        }
        Some(self.rate_delta_buf.iter().fold(0.0, |a, b| a + b) / self.rate_delta_buf.len() as f64)
    }
    pub fn maybe_fire_escalation_toast(&mut self) {
        if self.escalation_toast_fired {
            return;
        }
        if self.faulty_trip_count < 3.0 {
            return;
        }
        self.escalation_toast_fired = true;
        self.show_msg("THE WIRES ARE ANGRY NOW", 2.4);
        if self.ac.is_some() {
            self.noise_burst(0.10, 220.0, 1.2, 0.15, "lowpass", None, 0.0, 0.0);
        }
    }

    // ------------------------------------------------------------ level decay
    pub fn level_decay(&self, n: Option<i32>) -> f64 {
        let lv = n.unwrap_or_else(|| self.depth()) as f64;
        clamp(DECAY_BASE + (lv - 1.0) * DECAY_STEP, DECAY_BASE, DECAY_MAX)
    }

    /// Regions with their wall / ceiling / floor variants, grime, light and
    /// the dark (blacked-out) share. Surfaces follow the areas of zones (see
    /// [`purpose`]) and the lights fail room by room (`region_units`), so
    /// both change at walls and doorways, never in the middle of a room.
    pub fn build_regions(&mut self) {
        let (w, h) = (self.w, self.h);
        let n = (w * h) as usize;
        self.region_map = vec![0; n];
        self.ceil_region = vec![0; n];
        let palette_count = WALL_PALETTES.len() as i64;
        let decay = self.level_decay(None);
        // shallow floors mostly land on the canonical wallpaper
        let canon_yellows = [PAL_MONO_YELLOW as i64, PAL_MONO_YELLOW as i64, PAL_MONO_YELLOW as i64, 9, 1, 0];
        let primary_pal = if rnd() > decay * 0.9 { choice(&canon_yellows) } else { irand(0, palette_count - 1) };
        let secondary_pal = if rnd() < 0.75 {
            (primary_pal + (if rnd() < 0.5 { 1 } else { -1 }) + palette_count) % palette_count
        } else {
            irand(0, palette_count - 1)
        };
        let level_palettes = [primary_pal, secondary_pal];
        let wv: Vec<(bool, usize, i64)> = self.tex.wallpaper_variants.iter().map(|v| (v.moss, v.palette_idx, v.grime_lvl)).collect();
        let pick_correlated = |p_idx: i64| -> RegionInfo {
            let r = rnd();
            // clean takes whatever the decay has not claimed; the remainder
            // splits across aged / dirty / ruined, weighted to the milder end
            let g: i64 = if r < 1.0 - decay {
                0
            } else if r < 1.0 - decay * 0.45 {
                1
            } else if r < 1.0 - decay * 0.12 {
                2
            } else {
                3
            };
            let mut candidates: Vec<usize> = vec![];
            for (i, &(moss, pal, gl)) in wv.iter().enumerate() {
                if !moss && pal as i64 == p_idx && gl == g {
                    candidates.push(i);
                }
            }
            if candidates.is_empty() {
                for (i, &(moss, pal, _)) in wv.iter().enumerate() {
                    if !moss && pal as i64 == p_idx {
                        candidates.push(i);
                    }
                }
            }
            let wall = if !candidates.is_empty() { choice(&candidates) } else { irand(0, wv.len() as i64 - 1).max(0) as usize };
            RegionInfo {
                wall,
                ceil: g.min(3) as usize,
                floor: g.min(3) as usize,
                flags: 0,
                grime: g as i32,
                lit: Some(GRIME_LIGHT[g as usize]),
            }
        };
        let RegionUnits { tile: unit_of, surface, keys } = self.region_units();
        let region_count = surface.len();
        // the surfaces: one palette per kind of place (every area of one
        // purpose is papered alike), grime by the decay
        let mut palette_of: Vec<(&'static str, i64)> = vec![];
        let mut surfaces: Vec<RegionInfo> = Vec::with_capacity(keys.len());
        for key in &keys {
            let p_idx = match key.and_then(|k| palette_of.iter().find(|(pk, _)| *pk == k)) {
                Some(&(_, p)) => p,
                None => {
                    let p = level_palettes[if rnd() < 0.7 { 0 } else { 1 }];
                    if let Some(k) = key {
                        palette_of.push((k, p));
                    }
                    p
                }
            };
            surfaces.push(pick_correlated(p_idx));
        }
        let spawn_region = unit_of[self.t_idx(1, 1)];
        let spawn_surface = surface[spawn_region];
        // the first thing you see is clean on the shallow floors, never ruined
        {
            let info = &mut surfaces[spawn_surface];
            let want_g = if rnd() > decay { 0 } else { info.grime.min(1) };
            if info.grime > want_g {
                let pal = wv[info.wall].1;
                let matched: Vec<usize> =
                    wv.iter().enumerate().filter(|(_, v)| !v.0 && v.1 == pal && v.2 == want_g as i64).map(|(i, _)| i).collect();
                if !matched.is_empty() {
                    info.wall = choice(&matched);
                }
                info.grime = want_g;
                info.ceil = want_g as usize;
                info.floor = want_g as usize;
                info.lit = Some(GRIME_LIGHT[want_g as usize]);
            }
        }
        // every region wears its surface; moss and the lights then take rooms
        let mut region_info: Vec<RegionInfo> = surface.iter().map(|&s| surfaces[s].clone()).collect();
        // moss: a symptom of rot; never on level 1, never where you arrive
        let moss_regions = if self.depth() > 1 && rnd() < decay * 0.9 {
            if rnd() < decay {
                2
            } else {
                1
            }
        } else {
            0
        };
        let others: Vec<usize> = (0..region_count).filter(|&r| r != spawn_region).collect();
        for _ in 0..moss_regions {
            let Some(r) = choice_opt(&others) else { break };
            let moss_walls: Vec<usize> = wv.iter().enumerate().filter(|(_, v)| v.0).map(|(i, _)| i).collect();
            let wall = choice_opt(&moss_walls).unwrap_or(0);
            region_info[r] = RegionInfo { wall, ceil: 4, floor: 4, flags: RF_MOSS, grime: 3, lit: Some(0.22) };
        }
        // ---- dark-region selection scaled by level ----
        let mut total_open = 0usize;
        let mut open_per_region = vec![0usize; region_count];
        for i in 0..n {
            if self.tiles[i] == 0 {
                open_per_region[unit_of[i]] += 1;
                total_open += 1;
            }
        }
        // blackout is the far end of the same rot: a share of the decay
        let dark_target = clamp(decay * 0.7, 0.05, 0.8);
        let mut candidates: Vec<usize> = vec![];
        for r in 0..region_count {
            if r == spawn_region {
                continue;
            }
            if region_info[r].flags & RF_MOSS != 0 {
                continue;
            }
            if open_per_region[r] == 0 {
                continue;
            }
            candidates.push(r);
        }
        // shuffle, then bring the grimiest regions forward (stable sort): the
        // lights fail first where it rotted
        let mut i = candidates.len();
        while i > 1 {
            i -= 1;
            let j = (rnd() * (i + 1) as f64).floor() as usize;
            candidates.swap(i, j);
        }
        candidates.sort_by(|&a, &b| region_info[b].grime.cmp(&region_info[a].grime));
        let mut dark_open = 0usize;
        let tot = total_open.max(1) as f64;
        for &r in &candidates {
            if dark_open as f64 / tot >= dark_target {
                break;
            }
            // stop short rather than blowing past the target (the first dark
            // region may overshoot further: a level of a few big areas has no
            // smaller one to give)
            let cap = if dark_open == 0 { 2.0 } else { 1.25 };
            if (dark_open + open_per_region[r]) as f64 / tot > dark_target * cap {
                continue;
            }
            region_info[r].flags |= RF_DARK;
            dark_open += open_per_region[r];
        }
        for i in 0..n {
            let id = unit_of[i];
            self.region_map[i] = id as u8;
            self.ceil_region[i] = region_info[id].ceil as u8;
        }
        self.region_info = region_info;
    }

    /// How the level is cut into regions: per tile its region, per region its
    /// surface, per surface the key of its palette. A region is a group of
    /// the carvers' rooms of one area that open into each other (the line
    /// between them mostly open), so the lights fail room by room; a surface
    /// (wallpaper, grime) is an area, or for a big lone zone the regions on
    /// either side of its most walled line. Both change only at walls and
    /// doorways. Without a layout both are Voronoi cells.
    fn region_units(&self) -> RegionUnits {
        let (w, h) = (self.w, self.h);
        let n = (w * h) as usize;
        let Some(l) = self.layout.as_ref() else {
            // no zones: Voronoi cells around random seeds, flooded over every tile
            let count = 4.max((w * h) / 65) as usize;
            let mut q: Vec<(i32, i32, usize)> =
                (0..count).map(|i| (irand(1, (w - 2) as i64) as i32, irand(1, (h - 2) as i64) as i32, i)).collect();
            let mut unit = vec![usize::MAX; n];
            for &(sx, sz, id) in &q {
                let t = self.t_idx(sx, sz);
                if unit[t] == usize::MAX {
                    unit[t] = id;
                }
            }
            let mut qi = 0;
            while qi < q.len() {
                let (cx, cz, id) = q[qi];
                for (dx, dz) in N4 {
                    let (nx, nz) = (cx + dx, cz + dz);
                    if self.in_grid(nx, nz) && unit[self.t_idx(nx, nz)] == usize::MAX {
                        let ni = self.t_idx(nx, nz);
                        unit[ni] = id;
                        q.push((nx, nz, id));
                    }
                }
                qi += 1;
            }
            let tile = unit.into_iter().map(|u| if u == usize::MAX { 0 } else { u }).collect();
            return RegionUnits { tile, surface: (0..count).collect(), keys: vec![None; count] };
        };
        let areas = purpose::zone_areas(l, &self.tiles);
        let area_of = |t: usize| areas[l.zone_of[t] as usize];
        // ---- regions: rooms of an area that open into each other are one
        let rooms = &l.rooms;
        let mut group: Vec<usize> = (0..rooms.len()).collect();
        fn root(g: &mut [usize], mut i: usize) -> usize {
            while g[i] != i {
                g[i] = g[g[i]];
                i = g[i];
            }
            i
        }
        for a in 0..rooms.len() {
            for b in a + 1..rooms.len() {
                if areas[rooms[a].zone] != areas[rooms[b].zone] {
                    continue;
                }
                if self.rooms_wall_share(&rooms[a], &rooms[b]).map(|s| s < 0.5).unwrap_or(false) {
                    let (ra, rb) = (root(&mut group, a), root(&mut group, b));
                    group[ra.max(rb)] = ra.min(rb);
                }
            }
        }
        let mut room_of = vec![usize::MAX; n];
        for (ri, r) in rooms.iter().enumerate() {
            for z in r.z0..=r.z1 {
                for x in r.x0..=r.x1 {
                    room_of[self.t_idx(x, z)] = ri;
                }
            }
        }
        // ids in tile order
        let mut ids: std::collections::BTreeMap<usize, usize> = Default::default();
        let mut unit_area: Vec<usize> = vec![];
        let mut unit = vec![usize::MAX; n];
        for t in 0..n {
            if room_of[t] != usize::MAX {
                let g = root(&mut group, room_of[t]);
                unit[t] = *ids.entry(g).or_insert_with(|| {
                    unit_area.push(area_of(t));
                    unit_area.len() - 1
                });
            }
        }
        // the lines between rooms (and the border) join a neighbouring room of their area
        let mut q: std::collections::VecDeque<usize> = (0..n).filter(|&t| unit[t] != usize::MAX).collect();
        while let Some(t) = q.pop_front() {
            let (x, z) = (t as i32 % w, t as i32 / w);
            for (dx, dz) in N4 {
                if self.in_grid(x + dx, z + dz) {
                    let ni = self.t_idx(x + dx, z + dz);
                    if unit[ni] == usize::MAX && area_of(ni) == area_of(t) {
                        unit[ni] = unit[t];
                        q.push_back(ni);
                    }
                }
            }
        }
        for t in 0..n {
            if unit[t] == usize::MAX {
                let key = usize::MAX - area_of(t);
                unit[t] = *ids.entry(key).or_insert_with(|| {
                    unit_area.push(area_of(t));
                    unit_area.len() - 1
                });
            }
        }
        // ---- surfaces: per area one; a big lone zone's regions on either side of its line
        let have_purposes = self.zone_purposes.len() == l.zones.len();
        let key_of = |zi: usize| -> &'static str {
            if have_purposes {
                self.zone_purposes[zi].primary.unwrap_or("")
            } else {
                l.zones[zi].arch
            }
        };
        // where each region lies along x and z (mean tile)
        let mut centre = vec![(0.0f64, 0.0f64, 0usize); unit_area.len()];
        for t in 0..n {
            let c = &mut centre[unit[t]];
            c.0 += (t as i32 % w) as f64;
            c.1 += (t as i32 / w) as f64;
            c.2 += 1;
        }
        let mut surface = vec![usize::MAX; unit_area.len()];
        let mut keys: Vec<Option<&'static str>> = vec![];
        for zi in 0..l.zones.len() {
            if areas[zi] != zi {
                continue;
            }
            let near = keys.len();
            keys.push(Some(key_of(zi)));
            let lone = (0..l.zones.len()).all(|k| k == zi || areas[k] != zi);
            let line = if lone { self.region_split_line(&l.zones[zi]) } else { None };
            let side = |u: usize| -> bool {
                let (sx, sz, k) = centre[u];
                match line {
                    Some((true, at)) => sx / k as f64 > at as f64,
                    Some((false, at)) => sz / k as f64 > at as f64,
                    None => false,
                }
            };
            let mine: Vec<usize> = (0..unit_area.len()).filter(|&u| unit_area[u] == zi).collect();
            let far_count = mine.iter().filter(|&&u| side(u)).count();
            let far = if far_count > 0 && far_count < mine.len() {
                keys.push(Some(key_of(zi)));
                Some(keys.len() - 1)
            } else {
                None
            };
            for u in mine {
                surface[u] = match far {
                    Some(f) if side(u) => f,
                    _ => near,
                };
            }
        }
        // (a region id is a byte: past that, a region per surface)
        if surface.len() > 255 {
            let tile = (0..n).map(|t| surface[unit[t]]).collect();
            return RegionUnits { tile, surface: (0..keys.len()).collect(), keys };
        }
        RegionUnits { tile: unit, surface, keys }
    }

    /// The share of wall on the line between two rooms that lie side by
    /// side (one partition line apart, or touching across a zone line);
    /// None when they do not share a side
    fn rooms_wall_share(&self, a: &archgen::Room, b: &archgen::Room) -> Option<f64> {
        let line = |along_z: bool, at: i32, lo: i32, hi: i32| -> f64 {
            let walls = (lo..=hi).filter(|&k| if along_z { self.is_wall_tile(at, k) } else { self.is_wall_tile(k, at) }).count();
            walls as f64 / (hi - lo + 1) as f64
        };
        let (z0, z1) = (a.z0.max(b.z0), a.z1.min(b.z1));
        if z0 <= z1 {
            let (west, east) = if a.x1 < b.x0 { (a, b) } else { (b, a) };
            match east.x0 - west.x1 {
                2 => return Some(line(true, west.x1 + 1, z0, z1)),
                1 => return Some(line(true, west.x1, z0, z1).max(line(true, east.x0, z0, z1))),
                _ => {}
            }
        }
        let (x0, x1) = (a.x0.max(b.x0), a.x1.min(b.x1));
        if x0 <= x1 {
            let (north, south) = if a.z1 < b.z0 { (a, b) } else { (b, a) };
            match south.z0 - north.z1 {
                2 => return Some(line(false, north.z1 + 1, x0, x1)),
                1 => return Some(line(false, north.z1, x0, x1).max(line(false, south.z0, x0, x1))),
                _ => {}
            }
        }
        None
    }

    /// Where a big thin-walled zone splits into two regions: the column
    /// (`true`) or row across its long axis, in its middle stretch, that is
    /// most wall (a partition with doorways). None for small zones or when
    /// no line is mostly wall.
    fn region_split_line(&self, z: &archgen::Zone) -> Option<(bool, i32)> {
        const SPLIT_AREA: i32 = 150;
        if z.wall != "thin" || z.arch == "store" || z.area() < SPLIT_AREA {
            return None;
        }
        let along_x = z.x1 - z.x0 >= z.z1 - z.z0;
        let (lo, hi, a, b) = if along_x { (z.x0, z.x1, z.z0, z.z1) } else { (z.z0, z.z1, z.x0, z.x1) };
        let len = hi - lo + 1;
        let mut best: Option<(bool, i32)> = None;
        let mut best_share = 0.6;
        for line in lo + len * 3 / 10..=hi - len * 3 / 10 {
            let walls = (a..=b).filter(|&k| if along_x { self.is_wall_tile(line, k) } else { self.is_wall_tile(k, line) }).count();
            let share = walls as f64 / (b - a + 1) as f64;
            if share > best_share {
                best_share = share;
                best = Some((along_x, line));
            }
        }
        best
    }
    /// (tile coords; a copy of the region's info)
    pub fn region_at(&self, x: i32, z: i32) -> RegionInfo {
        if self.region_info.is_empty() {
            return RegionInfo::default();
        }
        if self.in_grid(x, z) {
            self.region_info[self.region_map[self.t_idx(x, z)] as usize].clone()
        } else {
            self.region_info[0].clone()
        }
    }
    pub fn region_flags_at(&self, x: i32, z: i32) -> u32 {
        if self.region_info.is_empty() || !self.in_grid(x, z) {
            return 0;
        }
        self.region_info[self.region_map[self.t_idx(x, z)] as usize].flags
    }
    /// (one `choice` draw) — position + outward normal against an adjacent wall
    pub fn wall_adj_spot(&self, x: i32, z: i32) -> Option<WallSpot> {
        let dirs: Vec<(i32, i32)> = N4.iter().copied().filter(|&(dx, dz)| self.is_wall_tile(x + dx, z + dz)).collect();
        if dirs.is_empty() {
            return None;
        }
        let (dx, dz) = choice(&dirs);
        let (fx, fz) = (dx as f64, dz as f64);
        Some(WallSpot {
            pos: Vec3::new(self.wx(x) + fx * (CELL / 2.0 - 0.06), 0.0, self.wz(z) + fz * (CELL / 2.0 - 0.06)),
            // -dx of a 0 is -0 (it decides atan2's sign)
            normal: Vec3::new(-fx, 0.0, -fz),
            yaw: (-fx).atan2(-fz),
        })
    }
}
