//! Arenas for the creatures of the redesign (`docs/ENTITIES.md`): a small
//! grid level with no solids but its walls (`GridNav`, installed as the
//! world queries), one creature, and a player the test moves by hand (its
//! gait sets what the creatures hear, as `update_player` would). Game time
//! (`perf_t`) advances with each step, as `frame` would.
#![allow(dead_code)]
use game::base::numeric::hypot2;
use game::entities::*;
use game::prelude::*;
use std::rc::Rc;

/// Grid versions of the world queries (walls only) and the grid's light.
pub struct GridNav {
    pub lit: Vec<f64>,
}
impl EntityNav for GridNav {
    fn bfs(&self, g: &Game, sx: i32, sz: i32) -> Vec<i32> {
        let (w, h) = (g.w, g.h);
        let mut dist = vec![-1i32; (w * h) as usize];
        if !g.in_grid(sx, sz) || g.is_wall_tile(sx, sz) {
            return dist;
        }
        let mut q = vec![g.t_idx(sx, sz)];
        dist[q[0]] = 0;
        let mut qi = 0;
        while qi < q.len() {
            let cur = q[qi];
            let (cx, cz) = ((cur as i32) % w, (cur as i32) / w);
            let d = dist[cur] + 1;
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (nx, nz) = (cx + dx, cz + dz);
                if g.in_grid(nx, nz) && g.tiles[g.t_idx(nx, nz)] == 0 && dist[g.t_idx(nx, nz)] == -1 {
                    dist[g.t_idx(nx, nz)] = d;
                    q.push(g.t_idx(nx, nz));
                }
            }
            qi += 1;
        }
        dist
    }
    fn los(&self, g: &Game, ax: f64, az: f64, bx: f64, bz: f64) -> bool {
        let (dx, dz) = (bx - ax, bz - az);
        let steps = (hypot2(dx, dz) / 0.35).ceil();
        let mut i = 1.0;
        while i < steps {
            let t = i / steps;
            if g.is_wall_tile(g.tx(ax + dx * t), g.tz(az + dz * t)) {
                return false;
            }
            i += 1.0;
        }
        true
    }
    fn circle_hits_wall(&self, g: &Game, px: f64, pz: f64, r: f64) -> bool {
        let (x0, x1, z0, z1) = (g.tx(px - r), g.tx(px + r), g.tz(pz - r), g.tz(pz + r));
        for z in z0..=z1 {
            for x in x0..=x1 {
                if !g.is_wall_tile(x, z) {
                    continue;
                }
                let cx = clamp(px, g.wx(x) - CELL / 2.0, g.wx(x) + CELL / 2.0);
                let cz = clamp(pz, g.wz(z) - CELL / 2.0, g.wz(z) + CELL / 2.0);
                if (px - cx) * (px - cx) + (pz - cz) * (pz - cz) < r * r {
                    return true;
                }
            }
        }
        false
    }
    fn collide_move(&self, g: &Game, p: &mut Vec3, dx: f64, dz: f64, r: f64) {
        if dx != 0.0 && !self.circle_hits_wall(g, p.x + dx, p.z, r) {
            p.x += dx;
        }
        if dz != 0.0 && !self.circle_hits_wall(g, p.x, p.z + dz, r) {
            p.z += dz;
        }
    }
    fn nav_tile_at(&self, g: &Game, px: f64, pz: f64) -> (i32, i32) {
        (g.tx(px), g.tz(pz))
    }
    fn ceil_at_world(&self, _g: &Game, _px: f64, _pz: f64) -> f64 {
        3.0
    }
    fn region_light_at(&self, g: &Game, x: i32, z: i32) -> f64 {
        if !g.in_grid(x, z) {
            return 0.5;
        }
        self.lit.get(g.t_idx(x, z)).copied().unwrap_or(0.5)
    }
}

/// An `n` × `n` room walled round, with walls where `wall(x, z)` says, lit
/// at `light` everywhere.
pub fn room(n: i32, light: f64, wall: impl Fn(i32, i32) -> bool) -> Game {
    room_lit(n, |_, _| light, wall)
}

/// The same with the light of each tile given by `light(x, z)`.
pub fn room_lit(n: i32, light: impl Fn(i32, i32) -> f64, wall: impl Fn(i32, i32) -> bool) -> Game {
    let mut g = Game::new_bare();
    g.w = n;
    g.h = n;
    g.tiles = (0..n * n)
        .map(|i| {
            let (x, z) = (i % n, i / n);
            if x == 0 || z == 0 || x == n - 1 || z == n - 1 || wall(x, z) {
                1
            } else {
                0
            }
        })
        .collect();
    g.tile_ceil = vec![3.0; (n * n) as usize];
    set_entity_nav(Some(Rc::new(GridNav { lit: (0..n * n).map(|i| light(i % n, i / n)).collect() })));
    g.state = "playing".into();
    g.god_mode = true;
    g.now_ms = 12345.0;
    g.level_num = 5 + game::level::building::BUILDING_FLOORS;
    // long past the level's first seconds (the grace is tested on its own)
    g.perf_t = 100.0;
    g.level_start_t = 0.0;
    g.player.sanity = SAN_MAX;
    g.player.stamina = STAM_MAX;
    g
}

/// Put a creature of `kind` at tile (x, z), facing `yaw`, awake.
pub fn creature(g: &mut Game, kind: &str, x: i32, z: i32, yaw: f64) -> usize {
    let ei = g.add_entity(kind, x, z, 0);
    let e = &mut g.entities[ei];
    e.freeze = 0.0;
    e.yaw = yaw;
    ei
}

/// How you move this step: still, crouched, walking or running.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Gait {
    Still,
    Crouch,
    Walk,
    Run,
}

impl Gait {
    pub fn speed(self) -> f64 {
        match self {
            Gait::Still => 0.0,
            Gait::Crouch => CROUCH_WALK,
            Gait::Walk => WALK,
            Gait::Run => SPRINT,
        }
    }
    /// What the player's own update makes of it (`player.rs`): how far it is heard
    pub fn audible(self) -> f64 {
        match self {
            Gait::Still => 0.0,
            Gait::Crouch => 3.5,
            Gait::Walk => 6.5,
            Gait::Run => 13.0,
        }
    }
}

/// One step of `dt`: you move toward (x, z) at `gait` (along the grid's
/// tiles, round walls), then the creatures update; game time advances.
pub fn step(g: &mut Game, dt: f64, toward: Option<(f64, f64)>, gait: Gait) {
    step_facing(g, dt, toward, gait, None);
}

/// The same, but you keep your eyes on `face` (walking backward, or
/// sideways, while you watch something), when given.
pub fn step_facing(g: &mut Game, dt: f64, toward: Option<(f64, f64)>, gait: Gait, face: Option<(f64, f64)>) {
    let speed = gait.speed();
    g.player.crouch = gait == Gait::Crouch;
    g.player.audible_range = if toward.is_some() { gait.audible() } else { 0.0 };
    if let (Some((tx, tz)), true) = (toward, speed > 0.0) {
        // the next tile along the shortest way, then straight on
        let nav = GridNav { lit: vec![] };
        let (gx, gz) = (g.tx(tx), g.tz(tz));
        let field = nav.bfs(g, gx, gz);
        let (px, pz) = (g.tx(g.player.pos.x), g.tz(g.player.pos.z));
        let here = field.get(g.t_idx(px, pz)).copied().unwrap_or(-1);
        let mut aim = (tx, tz);
        if here > 1 {
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (nx, nz) = (px + dx, pz + dz);
                if g.in_grid(nx, nz) && field[g.t_idx(nx, nz)] == here - 1 {
                    aim = (g.wx(nx), g.wz(nz));
                    break;
                }
            }
        }
        let (dx, dz) = (aim.0 - g.player.pos.x, aim.1 - g.player.pos.z);
        let d = hypot2(dx, dz);
        if d > 1e-6 {
            let s = (speed * dt).min(d);
            let mut p = g.player.pos;
            nav.collide_move(g, &mut p, dx / d * s, dz / d * s, PLAYER_R);
            g.player.pos = p;
            g.player.yaw = (-dx).atan2(-dz);
        }
    }
    if let Some((x, z)) = face {
        look_at(g, x, z);
    }
    g.perf_t += dt;
    g.now_ms += dt * 1000.0;
    g.update_entities(dt, g.perf_t);
}

/// You look at (x, z) (the yaw that faces it).
pub fn look_at(g: &mut Game, x: f64, z: f64) {
    g.player.yaw = (-(x - g.player.pos.x)).atan2(-(z - g.player.pos.z));
}

/// The mode of creature `ei`.
pub fn mode(g: &Game, ei: usize) -> game::entities::brain::Mode {
    g.creature(ei).map(|c| c.brain.mode).unwrap_or_default()
}

/// The modes creature `ei` has been through, in order (from its log).
pub fn modes(g: &Game, ei: usize) -> Vec<game::entities::brain::Mode> {
    g.creature(ei).map(|c| c.brain.log.iter().map(|x| x.2).collect()).unwrap_or_default()
}
