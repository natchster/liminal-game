//! A synthetic level for smoke-testing placers without the full game:
//! a walled grid with rooms, partitions and a BFS distance field from (1,1).

use crate::base::game::Game;
use crate::base::rng::{rnd, with_seed};
use engine::math::Vec3;

impl Game {
    /// A W×H (odd) grid: border walls, a lattice of wall stubs and random
    /// interior walls, every open tile reachable. Returns the game and `dist`.
    pub fn test_level(seed: u32, size: i32) -> (Game, Vec<i32>) {
        let mut g = Game::new_bare();
        let n = if size % 2 == 0 { size + 1 } else { size };
        g.w = n;
        g.h = n;
        g.tiles = vec![0; (n * n) as usize];
        with_seed(seed, || {
            for z in 0..n {
                for x in 0..n {
                    let border = x == 0 || z == 0 || x == n - 1 || z == n - 1;
                    let lattice = x % 4 == 0 && z % 4 == 0;
                    let i = (z * n + x) as usize;
                    if border || lattice || (x > 2 && z > 2 && rnd() < 0.12) {
                        g.tiles[i] = 1;
                    }
                }
            }
        });
        g.tiles[(n + 1) as usize] = 0;
        let dist = g.test_dist();
        // seal unreachable pockets
        for i in 0..g.tiles.len() {
            if g.tiles[i] == 0 && dist[i] < 0 {
                g.tiles[i] = 1;
            }
        }
        let dist = g.test_dist();
        g.player.pos = Vec3::new(g.wx(1), 0.0, g.wz(1));
        g.has_layout = false;
        g.level_weather = &crate::base::state::WEATHER_BIASES[(seed as usize) % 6];
        (g, dist)
    }
    /// A small floor of the Building, by hand (9 × 5 tiles): a gallery at 0,
    /// a flight down to a gallery at −3 along +x, a void between the two
    /// rows, and a wall standing between the two heights on the bottom row.
    ///
    /// ```text
    ///   # # # # # # # # #
    ///   # . . . S , , , #    S: the flight, 0 → −3 toward +x
    ///   # . ~ ~ ~ ~ ~ , #    ~: void
    ///   # . . . # , , , #    #: wall (a cliff between the heights)
    ///   # # # # # # # # #
    /// ```
    pub fn test_ground_level() -> Game {
        use crate::level::archgen::{build_ground_solids, Ground};
        let mut g = Game::new_bare();
        let rows = ["#########", "#...S,,,#", "#.~~~~~,#", "#...#,,,#", "#########"];
        g.w = 9;
        g.h = 5;
        g.tiles = vec![1; 45];
        g.ground = vec![Ground::void(); 45];
        for (z, row) in rows.iter().enumerate() {
            for (x, c) in row.chars().enumerate() {
                let i = z * 9 + x;
                g.ground[i] = match c {
                    '.' => Ground::floor(0.0),
                    ',' => Ground::floor(-3.0),
                    'S' => Ground::stair(0.0, -3.0, 0),
                    '~' => Ground::void(),
                    _ => Ground::mass(-30.0, 30.0),
                };
                if g.ground[i].walkable() {
                    g.tiles[i] = 0;
                }
            }
        }
        g.tile_ceil = vec![60.0; 45];
        g.solids = Some(build_ground_solids(9, 5, crate::base::config::CELL, &g.ground));
        g.player.pos = Vec3::new(g.wx(2), crate::base::config::EYE, g.wz(1));
        g.has_layout = false;
        g.level_weather = &crate::base::state::WEATHER_BIASES[0];
        g
    }
    /// BFS tile distance from (1, 1); -1 where unreachable or wall.
    pub fn test_dist(&self) -> Vec<i32> {
        let (w, h) = (self.w, self.h);
        let mut d = vec![-1; (w * h) as usize];
        let mut q = std::collections::VecDeque::new();
        let s = (w + 1) as usize;
        if self.tiles[s] != 0 {
            return d;
        }
        d[s] = 0;
        q.push_back((1, 1));
        while let Some((x, z)) = q.pop_front() {
            let dd = d[(z * w + x) as usize];
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (nx, nz) = (x + dx, z + dz);
                if nx < 0 || nz < 0 || nx >= w || nz >= h {
                    continue;
                }
                let i = (nz * w + nx) as usize;
                if self.tiles[i] != 0 || d[i] >= 0 {
                    continue;
                }
                d[i] = dd + 1;
                q.push_back((nx, nz));
            }
        }
        d
    }
    /// Run every registered animator once (the main loop does this each frame).
    pub fn run_animators(&mut self, dt: f64, t: f64) {
        let mut anims = std::mem::take(&mut self.animators);
        for a in anims.iter_mut() {
            a(self, dt, t);
        }
        anims.append(&mut self.animators);
        self.animators = anims;
    }
    /// Use every enabled interactable (smoke tests).
    pub fn use_all_interactables(&mut self) {
        for i in 0..self.interactables.len() {
            if !(self.interactables[i].enabled)(self) {
                continue;
            }
            let mut f = std::mem::replace(&mut self.interactables[i].on_use, Box::new(|_| {}));
            f(self);
            if i < self.interactables.len() {
                self.interactables[i].on_use = f;
            }
        }
    }
}

/// Smoke-test a list of placers: several seeds and weathers, a few animator
/// frames and every interactable used once. Panics surface as test failures.
pub fn smoke_placers(list: &[(&'static str, crate::base::state::PlacerFn)]) {
    for seed in 0..12u32 {
        let (mut g, dist) = Game::test_level(seed * 7919 + 13, 21 + (seed as i32 % 3) * 8);
        crate::base::rng::with_seed(seed.wrapping_mul(2654435761), || {
            // many runs per seed so rare branches (e.g. an early return unless `rnd()` >= 0.95) execute
            for rep in 0..24 {
                for (name, f) in list {
                    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(&mut g, &dist)));
                    if r.is_err() {
                        panic!("placer {name} panicked (seed {seed}, run {rep})");
                    }
                }
            }
        });
        for k in 0..30 {
            let t = k as f64 * 0.37;
            g.perf_t = t;
            g.run_animators(1.0 / 60.0, t);
        }
        g.player.flashlight_on = true;
        g.use_all_interactables();
        for k in 0..10 {
            g.run_animators(0.25, 20.0 + k as f64);
        }
        g.world.update_matrix_world(true);
    }
}
