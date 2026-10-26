//! The story layer: M.'s trail and her camp on every floor (and her
//! notebook, `notebook`). The design is `docs/GAMEPLAY.md`; the canon is
//! `docs/LORE.md`, *M.'s descent*.
//!
//! On each level M. came in where you did, camped somewhere off the straight
//! way, and went on to the door, leaving sheets of paper folded twice with a
//! pencil arrow on each. `plan_story` decides, from the level as built, where
//! her camp is, the walk she took (arrival → camp → door, along the floor
//! plan) and which tiles of it got a sheet; `place_story` dresses it.
//!
//! Placement runs inside the level build once settling has made the walls
//! final (its sight lines must hold in the finished level) and before
//! placement (so the props round her things make room for them), and draws
//! only from its own salted stream (`STORY_SALT`), never from `rnd()`: no
//! build checkpoint moves. What depends on the run's progress (whether her
//! page here was already taken) is fitted after the build, like the notes.
//!
//! The trail is meant to be read, not followed blindly: a sheet is placed
//! where the last one can be seen from, at most `spacing` tiles of walk on,
//! and from level `GAP_FROM` one stretch after the camp has lost its sheets
//! (two from level 7). The sheets get sparser as you go down, as she did.
pub mod dress;

use crate::prelude::*;

/// Salt of the stream the story layer draws from
pub const STORY_SALT: u64 = 0x5703_7EA1_C0FF_EE11;
/// From this level, a stretch of the trail after the camp has lost its sheets
pub const GAP_FROM: i32 = 4;
/// How far the sheets lie from the centre of their tile at most, m
const JITTER: f64 = 0.35;
/// The fixed order neighbouring tiles are tried in (N, E, S, W)
const DIRS: [(i32, i32); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];

/// Tiles of walk between two of her sheets at most, by level: she left them
/// often at first, then less.
pub fn spacing(level: i32) -> usize {
    match level {
        ..=2 => 3,
        3..=5 => 4,
        _ => 5,
    }
}

/// Her camp on this floor.
#[derive(Clone, Debug)]
pub struct Camp {
    pub tile: (i32, i32),
    /// The side of the tile her wall is on (one of `DIRS`)
    pub wall: (i32, i32),
    /// The camp's centre on the floor, and the way out from her wall
    pub at: Vec3,
    pub normal: Vec3,
    /// Her lamp, and its light
    pub lamp: Option<Obj>,
    pub light: Option<Obj>,
    /// Whether her lamp still burns
    pub lit: bool,
}

/// One of her sheets.
#[derive(Clone, Debug)]
pub struct TrailMark {
    /// Its index in the walk
    pub step: usize,
    pub x: f64,
    pub z: f64,
    /// The way its arrow points (a yaw: forward is (−sin, −cos))
    pub yaw: f64,
    pub g: Option<Obj>,
}

/// The story layer of the level being played.
#[derive(Clone, Debug, Default)]
pub struct StoryLevel {
    pub camp: Option<Camp>,
    /// The walk she took, arrival → camp → door, tile by tile
    pub walk: Vec<(i32, i32)>,
    /// Where the camp is in the walk
    pub camp_step: usize,
    pub trail: Vec<TrailMark>,
    /// Stretches of the walk (from step, to step) whose sheets are gone
    pub gaps: Vec<(usize, usize)>,
}

/// What `plan_story` decides: pure data, no objects.
#[derive(Clone, Debug, PartialEq)]
pub struct StoryPlan {
    pub camp: (i32, i32),
    pub wall: (i32, i32),
    pub walk: Vec<(i32, i32)>,
    pub camp_step: usize,
    /// (walk step, x, z, yaw) of each sheet
    pub marks: Vec<(usize, f64, f64, f64)>,
    pub gaps: Vec<(usize, usize)>,
}

impl Game {
    /// Whether a body can walk through tile (x, z) on her way: open floor,
    /// not barricaded, no hole in it.
    fn story_open(&self, x: i32, z: i32) -> bool {
        if self.is_wall_tile(x, z) {
            return false;
        }
        let i = (z * self.w + x) as usize;
        if self.barricade_tiles.contains(&i) {
            return false;
        }
        !self.floor_holes.iter().any(|h| self.tx(h.x) == x && self.tz(h.z) == z)
    }

    /// Steps of walk from tile `from` to every tile (−1: not reachable).
    pub fn story_walk_dist(&self, from: (i32, i32)) -> Vec<i32> {
        let (w, h) = (self.w, self.h);
        let mut d = vec![-1i32; (w * h).max(0) as usize];
        if !self.in_grid(from.0, from.1) || !self.story_open(from.0, from.1) {
            return d;
        }
        let mut q = std::collections::VecDeque::new();
        d[(from.1 * w + from.0) as usize] = 0;
        q.push_back(from);
        while let Some((x, z)) = q.pop_front() {
            let k = d[(z * w + x) as usize];
            for (dx, dz) in DIRS {
                let (nx, nz) = (x + dx, z + dz);
                if nx < 0 || nz < 0 || nx >= w || nz >= h || !self.story_open(nx, nz) {
                    continue;
                }
                let i = (nz * w + nx) as usize;
                if d[i] < 0 {
                    d[i] = k + 1;
                    q.push_back((nx, nz));
                }
            }
        }
        d
    }

    /// The walk from `from` down the distances `d` to where they are 0, tile
    /// by tile (neighbours tried in the fixed order `DIRS`).
    fn story_descend(&self, d: &[i32], from: (i32, i32)) -> Vec<(i32, i32)> {
        let mut out = vec![from];
        let mut cur = from;
        let at = |p: (i32, i32)| d[(p.1 * self.w + p.0) as usize];
        while at(cur) > 0 {
            let next = DIRS.iter().map(|(dx, dz)| (cur.0 + dx, cur.1 + dz)).find(|&(x, z)| self.in_grid(x, z) && at((x, z)) == at(cur) - 1);
            let Some(n) = next else { break };
            out.push(n);
            cur = n;
        }
        out
    }

    /// Where her wall's face is on the side `side` of tile `tile`: walking
    /// from the tile's centre toward it until a body would touch it. None
    /// when there is no wall there, or it is too narrow to camp against (a
    /// pillar, a jamb): her bedroll and her count of days need a wall the
    /// length of a body behind them.
    pub fn camp_face(&self, tile: (i32, i32), side: (i32, i32)) -> Option<Vec3> {
        let (cx, cz) = (self.wx(tile.0), self.wz(tile.1));
        let (wx, wz) = (side.0 as f64, side.1 as f64);
        let mut s = 0.0;
        while !self.circle_hits_wall(cx + wx * s, cz + wz * s, 0.02) {
            s += 0.05;
            if s > CELL * 0.75 {
                return None;
            }
        }
        // the wall must be there a step to either side too
        let (ax, az) = (cx + wx * (s + 0.06), cz + wz * (s + 0.06));
        let wide = [-1.0, 1.0].iter().all(|k| self.circle_hits_wall(ax - wz * 1.05 * k, az + wx * 1.05 * k, 0.02));
        wide.then(|| v3(cx + wx * s, 0, cz + wz * s))
    }

    /// Where M. camped on this level and the walk she took: pure, from the
    /// level as built and her salted stream. None without an exit or a way
    /// to it.
    pub fn plan_story(&self) -> Option<StoryPlan> {
        let e = self.exit_data.as_ref()?;
        let exit = (self.tx(e.pos.x), self.tz(e.pos.z));
        let spawn = (self.tx(self.spawn_pos.x), self.tz(self.spawn_pos.z));
        let ds = self.story_walk_dist(spawn);
        let de = self.story_walk_dist(exit);
        let idx = |p: (i32, i32)| (p.1 * self.w + p.0) as usize;
        let len = ds[idx(exit)];
        if len < 2 {
            return None;
        }
        let mut r = SplitMix64::new(self.current_seed, STORY_SALT);
        let walls_beside = |x: i32, z: i32| DIRS.iter().filter(|(dx, dz)| self.is_wall_tile(x + dx, z + dz)).count();
        // her camp: well away from both ends, off the straight way but not
        // far off it, against a wall (in a corner or a dead end if she could)
        let pick = |min_detour: i32| -> Vec<((i32, i32), usize)> {
            let mut c = vec![];
            for z in 0..self.h {
                for x in 0..self.w {
                    let (a, b) = (ds[idx((x, z))], de[idx((x, z))]);
                    if a < 0 || b < 0 {
                        continue;
                    }
                    let detour = a + b - len;
                    if a < 3.max(len * 3 / 10) || b < 4.max(len / 4) || detour < min_detour || detour > 12.max(len / 6) {
                        continue;
                    }
                    let walls = walls_beside(x, z);
                    if walls > 0 && DIRS.iter().any(|&d| self.camp_face((x, z), d).is_some()) {
                        c.push(((x, z), walls));
                    }
                }
            }
            c
        };
        let mut cands = pick(4);
        if cands.is_empty() {
            cands = pick(2);
        }
        let camp = if cands.is_empty() {
            // a level too small to stray in: she stopped halfway
            let walk = self.story_descend(&ds, exit);
            walk[walk.len() / 2]
        } else {
            let best = cands.iter().map(|c| c.1.min(2)).max().unwrap_or(1);
            let pool: Vec<(i32, i32)> = cands.iter().filter(|c| c.1.min(2) == best).map(|c| c.0).collect();
            pool[((r.next() * pool.len() as f64) as usize).min(pool.len() - 1)]
        };
        let mut sides: Vec<(i32, i32)> = DIRS.iter().copied().filter(|&d| self.camp_face(camp, d).is_some()).collect();
        if sides.is_empty() {
            sides = DIRS.iter().copied().filter(|(dx, dz)| self.is_wall_tile(camp.0 + dx, camp.1 + dz)).collect();
        }
        let wall = if sides.is_empty() { (0, -1) } else { sides[((r.next() * sides.len() as f64) as usize).min(sides.len() - 1)] };

        // the walk: arrival → camp (down the arrival's distances, reversed),
        // then camp → door
        let mut walk = self.story_descend(&ds, camp);
        walk.reverse();
        let camp_step = walk.len() - 1;
        walk.extend(self.story_descend(&de, camp).into_iter().skip(1));

        // the sheets: each where the last can be seen from, at most `spacing`
        // steps on; never on the camp's tile (the camp is its own mark) or the
        // door's
        let k = spacing(self.level_num);
        let n = walk.len();
        // where a sheet on `step` lies (the camp's step: the middle of its
        // tile, where the camp is seen from)
        let pos = |step: usize, r: &mut SplitMix64| {
            let (x, z) = walk[step];
            if step == camp_step {
                return (self.wx(x), self.wz(z));
            }
            (self.wx(x) + r.range(-JITTER, JITTER), self.wz(z) + r.range(-JITTER, JITTER))
        };
        let mut marks: Vec<(usize, f64, f64, f64)> = vec![];
        let first = 2.min(n - 1);
        let (mut px, mut pz) = pos(first, &mut r);
        let mut cur = first;
        let mut steps = vec![(first, px, pz)];
        while cur + 2 < n {
            let mut j = (cur + k).min(n - 2);
            if cur < camp_step && j > camp_step {
                j = camp_step;
            }
            // the furthest step in reach that this sheet can be seen from
            let mut placed = None;
            while j > cur {
                let mut rr = r;
                let (x, z) = pos(j, &mut rr);
                if self.los(px, pz, x, z) {
                    r = rr;
                    placed = Some((j, x, z));
                    break;
                }
                j -= 1;
            }
            let (j, x, z) = placed.unwrap_or_else(|| {
                let (x, z) = pos(cur + 1, &mut r);
                (cur + 1, x, z)
            });
            steps.push((j, x, z));
            (px, pz, cur) = (x, z, j);
        }
        for &(step, x, z) in &steps {
            if step == camp_step {
                continue;
            }
            // the arrow points along the walk, two tiles on
            let ahead = walk[(step + 2).min(n - 1)];
            let (dx, dz) = (self.wx(ahead.0) - x, self.wz(ahead.1) - z);
            let yaw = (-dx).atan2(-dz) + r.range(-0.12, 0.12);
            marks.push((step, x, z, yaw));
        }

        // deeper down, a stretch after the camp has lost its sheets
        let mut gaps = vec![];
        if self.level_num >= GAP_FROM {
            let lose = if self.level_num >= 7 { 2 } else { 1 };
            // sheets after the camp, not the last two (by the door)
            let after: Vec<usize> = (0..marks.len()).filter(|&i| marks[i].0 > camp_step).collect();
            if after.len() >= lose + 3 {
                let room = after.len() - lose - 2;
                let at = after[(r.next() * room as f64) as usize % room.max(1)];
                // the mark the stretch starts from: a sheet, or her camp
                let prev = marks.iter().rev().find(|m| m.0 < marks[at].0).map(|m| m.0).unwrap_or(camp_step);
                let before = if camp_step < marks[at].0 { prev.max(camp_step) } else { prev };
                let gone: Vec<usize> = (at..at + lose).collect();
                let resume = marks[at + lose].0;
                gaps.push((before, resume));
                for i in gone.into_iter().rev() {
                    marks.remove(i);
                }
            }
        }
        Some(StoryPlan { camp, wall, walk, camp_step, marks, gaps })
    }

    /// Dress the level with her camp and her trail (`plan_story`). Called by
    /// the level build after the themed placers.
    pub fn place_story(&mut self) {
        self.story = StoryLevel::default();
        // `#nostory`: the level without her, for comparisons
        if self.has_flag("nostory") {
            return;
        }
        let Some(plan) = self.plan_story() else { return };
        let c0 = self.world.child_count();
        let mut r = SplitMix64::new(self.current_seed, STORY_SALT ^ 0xCA4F);
        let camp = self.dress_camp(&plan, &mut r);
        let mut trail = vec![];
        let sheet = self.trail_sheet_parts();
        for &(step, x, z, yaw) in &plan.marks {
            let g = self.dress_sheet(&sheet, x, z, yaw);
            trail.push(TrailMark { step, x, z, yaw, g: Some(g) });
        }
        self.story = StoryLevel { camp: Some(camp), walk: plan.walk, camp_step: plan.camp_step, trail, gaps: plan.gaps };
        // placed after settling, which brings every other prop's world matrix
        // up to date; the prop cull reads them (a stale one at the origin
        // hides the thing as far away)
        for k in c0..self.world.child_count() {
            if let Some(o) = self.world.child(k) {
                o.update_world_matrix(true, true);
            }
        }
    }
}
