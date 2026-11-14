//! The story layer: M.'s trail and her camp on every floor (and her
//! notebook, `notebook`). The design is `docs/design/GAMEPLAY.md`; the canon is
//! `docs/design/LORE.md`, *M.'s descent*.
//!
//! On each level M. came in where you did, camped somewhere off the straight
//! way, and went on to the door, marking the way: sheets of paper folded
//! twice with a pencil arrow on each, and as her paper ran short, what she
//! had (`marks`). `plan_story` decides, from the level as built, where her
//! camp is, the walk she took (arrival → camp → door, along the floor plan)
//! and which tiles of it got a mark; `place_story` dresses it.
//!
//! Placement runs inside the level build once settling has made the walls
//! final (its sight lines must hold in the finished level) and before
//! placement (so the props round her things make room for them), and draws
//! only from its own salted stream (`STORY_SALT`), never from `rnd()`: no
//! build checkpoint moves. What depends on the run's progress (whether her
//! page here was already taken) is fitted after the build, like the notes.
//!
//! Not every trail is hers: from the floor of her page that says so (`f6`),
//! a trail of flat sheets leaves hers and goes into a dead end, and the
//! place is waiting there (`plan_false_trails`, `story_lure`). Two from the
//! floor of `f7`. Hers are folded twice.
//!
//! The trail is meant to be read, not followed blindly: a mark is placed
//! where the last one can be seen from, at most `spacing` tiles of walk on
//! (a small one nearer: `MarkKind::reach`), and from level `GAP_FROM` one
//! stretch after the camp has lost its marks (two from level 7). They get
//! sparser as you go down, as she did.
pub mod dress;
pub mod ending;
pub mod marks;
pub mod notebook;

pub use marks::{MarkKind, PlanMark};

use crate::prelude::*;

/// Salt of the stream the story layer draws from
pub const STORY_SALT: u64 = 0x5703_7EA1_C0FF_EE11;
/// How close to a false trail's dead end the place notices you, m
pub const LURE_REACH: f64 = 2.5;
/// From this level, a stretch of the trail after the camp has lost its sheets
pub const GAP_FROM: i32 = 4;
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
    /// Her lamp's flame
    pub flame: Option<Obj>,
    /// Whether her lamp still burns; how much of its light is left while it
    /// gutters out (1 … 0), and whether it is guttering
    pub lit: bool,
    pub fade: f64,
    pub dimming: bool,
    /// Her page here (`notebook`), lying on the bedroll, and whether it was taken
    pub page: Option<Obj>,
    pub page_id: Option<&'static str>,
    pub page_taken: bool,
}

/// One of her marks (or a flat page of a trail that is not hers).
#[derive(Clone, Debug)]
pub struct TrailMark {
    /// Its index in the walk
    pub step: usize,
    pub kind: MarkKind,
    /// Where it is: on the floor, or on a wall's face (`normal` out of the
    /// wall, `y` up from the floor)
    pub x: f64,
    pub z: f64,
    pub y: f64,
    /// The way it shows (a yaw: forward is (−sin, −cos))
    pub yaw: f64,
    pub normal: (f64, f64),
    pub g: Option<Obj>,
}

impl TrailMark {
    /// A mark as planned, and what was made of it
    pub fn of(m: &PlanMark, g: Option<Obj>) -> Self {
        TrailMark { step: m.step, kind: m.kind, x: m.x, z: m.z, y: m.y, yaw: m.yaw, normal: m.normal, g }
    }
    /// Where it is seen from in its sight lines (`PlanMark::seen_at`)
    pub fn seen_at(&self) -> (f64, f64) {
        (self.x + self.normal.0 * marks::SEEN_OFF, self.z + self.normal.1 * marks::SEEN_OFF)
    }
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
    /// Trails that are not hers (`plan_false_trails`)
    pub false_trails: Vec<FalseTrail>,
    /// You have come near her camp on this floor (your asides are told once)
    pub camp_come: bool,
}

/// Within this of her camp (and on its storey) you have come upon it, m
pub const CAMP_NEAR: f64 = 3.5;

impl crate::base::game::Game {
    /// You come upon her camp: within `CAMP_NEAR` of it, on its storey. Your
    /// asides are told once a floor (`Use::Found`; once a run, they weigh it)
    pub fn update_camp_come(&mut self) {
        if self.story.camp_come {
            return;
        }
        let Some(at) = self.story.camp.as_ref().map(|c| c.at) else { return };
        let p = self.player.pos;
        if crate::base::numeric::hypot2(p.x - at.x, p.z - at.z) < CAMP_NEAR && (p.y - at.y).abs() < 2.5 {
            self.story.camp_come = true;
            self.aside(crate::audio::asides::Use::Found, "camp");
        }
    }
}

/// A trail that is not hers, as planned: the walk (its first tile on hers)
/// and its flat pages.
#[derive(Clone, Debug, PartialEq)]
pub struct FalsePlan {
    pub walk: Vec<(i32, i32)>,
    pub marks: Vec<PlanMark>,
}

/// A trail that is not hers, on the level: where it goes, its sheets, its
/// dead end, and whether you have stood there (the place answers once).
#[derive(Clone, Debug)]
pub struct FalseTrail {
    pub walk: Vec<(i32, i32)>,
    pub trail: Vec<TrailMark>,
    pub end: (f64, f64),
    pub sprung: bool,
}

/// What `plan_story` decides: pure data, no objects.
#[derive(Clone, Debug, PartialEq)]
pub struct StoryPlan {
    pub camp: (i32, i32),
    pub wall: (i32, i32),
    pub walk: Vec<(i32, i32)>,
    pub camp_step: usize,
    /// Her marks, in the order of the walk
    pub marks: Vec<PlanMark>,
    pub gaps: Vec<(usize, usize)>,
    pub false_trails: Vec<FalsePlan>,
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
        self.story_walk_dist_avoiding(from, &Default::default())
    }

    /// `story_walk_dist` that never steps onto the tiles in `avoid`.
    pub fn story_walk_dist_avoiding(&self, from: (i32, i32), avoid: &std::collections::HashSet<(i32, i32)>) -> Vec<i32> {
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
                if nx < 0 || nz < 0 || nx >= w || nz >= h || !self.story_open(nx, nz) || avoid.contains(&(nx, nz)) {
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
        // on the Building's floors her wall is building, never the edge of the void
        if !self.ground.is_empty() {
            let i = ((tile.1 + side.1) * self.w + tile.0 + side.0) as usize;
            if self.ground.get(i).is_none_or(|g| g.kind != crate::level::archgen::GROUND_MASS) {
                return None;
            }
        }
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
        wide.then(|| v3(cx + wx * s, self.floor_y_of(tile.0, tile.1), cz + wz * s))
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
        // (on the Building's floors she camped in a cell: the shelter there)
        let cells: Vec<(i32, i32)> = self.building.as_ref().map_or_default(|b| b.cells.iter().map(|c| c.tile).collect());
        let pick = |min_detour: i32| -> Vec<((i32, i32), usize)> {
            let mut c = vec![];
            for z in 0..self.h {
                for x in 0..self.w {
                    if self.building.is_some() && !cells.contains(&(x, z)) {
                        continue;
                    }
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
        // in a cell, against a side wall (its bench is at the back)
        if let Some(c) = self.building.as_ref().and_then(|b| b.cells.iter().find(|c| c.tile == camp)) {
            let back = crate::level::archgen::DIRS[c.door ^ 1];
            if sides.len() > 1 {
                sides.retain(|&d| d != (back[0], back[1]));
            }
        }
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
        let k = spacing(self.depth());
        let n = walk.len();
        // at the last camp her paper goes no further than her camp
        let last = if self.level_num == notebook::LAST_LEVEL { camp_step } else { n.saturating_sub(2) };
        let mut marks = self.lay_marks(&walk, 2.min(n - 1), last, Some(camp_step), k, true, &mut r);

        // deeper down, a stretch after the camp has lost its sheets
        let mut gaps = vec![];
        if self.depth() >= GAP_FROM {
            let lose = if self.depth() >= 7 { 2 } else { 1 };
            // sheets after the camp, not the last two (by the door)
            let after: Vec<usize> = (0..marks.len()).filter(|&i| marks[i].step > camp_step).collect();
            if after.len() >= lose + 3 {
                let room = after.len() - lose - 2;
                let at = after[(r.next() * room as f64) as usize % room.max(1)];
                // the mark the stretch starts from: a mark, or her camp
                let prev = marks.iter().rev().find(|m| m.step < marks[at].step).map_or(camp_step, |m| m.step);
                let before = if camp_step < marks[at].step { prev.max(camp_step) } else { prev };
                let gone: Vec<usize> = (at..at + lose).collect();
                let resume = marks[at + lose].step;
                gaps.push((before, resume));
                for i in gone.into_iter().rev() {
                    marks.remove(i);
                }
            }
        }
        let false_trails = self.plan_false_trails(&walk, &de, &mut r);
        Some(StoryPlan { camp, wall, walk, camp_step, marks, gaps, false_trails })
    }

    /// Her page on this level, if she left one (`notebook::page_on`).
    pub fn story_page(&self) -> Option<&'static notebook::Page> {
        notebook::page_on(self.run_seed, self.level_num, self.level_scp.as_deref())
    }

    /// How many trails that are not hers lie on this level: none above the
    /// floor of her page that says so (`f6`), one from there, two from the
    /// floor of `f7` on.
    pub fn false_trails_due(&self) -> usize {
        let book = notebook::notebook(self.run_seed);
        let from = |id: &str| book.iter().find(|(_, p)| p.id == id).map_or(i32::MAX, |(n, _)| *n);
        if self.level_num >= from("f7") {
            2
        } else if self.level_num >= from("f6") {
            1
        } else {
            0
        }
    }

    /// The trails that are not hers (`false_trails_due`): each leaves her
    /// walk at a tile with open floor beside it that is off her way, and
    /// goes 6–16 tiles into a dead end (three walls round it) well away from
    /// the door, never back onto her walk or onto another such trail. Flat
    /// sheets, laid like hers.
    fn plan_false_trails(&self, walk: &[(i32, i32)], de: &[i32], r: &mut SplitMix64) -> Vec<FalsePlan> {
        let mut out = vec![];
        let mut used: std::collections::HashSet<(i32, i32)> = walk.iter().copied().collect();
        let idx = |p: (i32, i32)| (p.1 * self.w + p.0) as usize;
        let walls_beside = |x: i32, z: i32| DIRS.iter().filter(|(dx, dz)| self.is_wall_tile(x + dx, z + dz)).count();
        for _ in 0..self.false_trails_due() {
            let mut branches = vec![];
            for i in 4..walk.len().saturating_sub(6) {
                for (dx, dz) in DIRS {
                    let t = (walk[i].0 + dx, walk[i].1 + dz);
                    if self.in_grid(t.0, t.1) && self.story_open(t.0, t.1) && !used.contains(&t) {
                        branches.push((i, t));
                    }
                }
            }
            for i in (1..branches.len()).rev() {
                branches.swap(i, ((r.next() * (i + 1) as f64) as usize).min(i));
            }
            for &(i, start) in branches.iter().take(40) {
                let d = self.story_walk_dist_avoiding(start, &used);
                let ends: Vec<(i32, i32)> = (0..self.h)
                    .flat_map(|z| (0..self.w).map(move |x| (x, z)))
                    .filter(|&t| (6..=16).contains(&d[idx(t)]) && de[idx(t)] >= 8 && walls_beside(t.0, t.1) >= 3)
                    .collect();
                if ends.is_empty() {
                    continue;
                }
                let end = ends[((r.next() * ends.len() as f64) as usize).min(ends.len() - 1)];
                let mut fw = self.story_descend(&d, end);
                fw.reverse();
                fw.insert(0, walk[i]);
                let marks = self.lay_marks(&fw, 1, fw.len() - 1, None, spacing(self.depth()), false, r);
                used.extend(fw.iter().copied());
                out.push(FalsePlan { walk: fw, marks });
                break;
            }
        }
        out
    }

    /// Dress the level with her camp and her trail (`plan_story`). Called by
    /// the level build after the themed placers.
    pub fn place_story(&mut self) {
        self.story = StoryLevel::default();
        // `#nostory`: the level without her, for comparisons; and below her
        // last camp her paper has ended (`ending`)
        if self.has_flag("nostory") || self.level_num > notebook::LAST_LEVEL {
            return;
        }
        let Some(plan) = self.plan_story() else { return };
        let c0 = self.world.child_count();
        let mut r = SplitMix64::new(self.current_seed, STORY_SALT ^ 0xCA4F);
        let mut camp = self.dress_camp(&plan, &mut r);
        if let Some(p) = self.story_page() {
            camp.page = Some(self.dress_page(&camp, &mut r));
            camp.page_id = Some(p.id);
            let at = v3(camp.at.x, camp.at.y + EYE, camp.at.z);
            self.interact(
                at,
                1.8,
                "[E] TAKE HER PAGE",
                |g| g.story.camp.as_ref().is_some_and(|c| c.page.is_some() && !c.page_taken),
                |g| g.take_page(),
            );
        }
        let mut trail = vec![];
        let mut kit = dress::MarkKit::default();
        for m in &plan.marks {
            let g = self.dress_mark(&mut kit, m, &mut r);
            trail.push(TrailMark::of(m, Some(g)));
        }
        // the trails that are not hers, and the place waiting at their ends
        let mut false_trails = vec![];
        for (k, f) in plan.false_trails.iter().enumerate() {
            let trail = f.marks.iter().map(|m| TrailMark::of(m, Some(self.dress_mark(&mut kit, m, &mut r)))).collect();
            let e = *f.walk.last().unwrap();
            let end = (self.wx(e.0), self.wz(e.1));
            false_trails.push(FalseTrail { walk: f.walk.clone(), trail, end, sprung: false });
            self.animate(move |g, _dt, _t| {
                let Some(ft) = g.story.false_trails.get(k) else { return };
                let (dx, dz) = (ft.end.0 - g.player.pos.x, ft.end.1 - g.player.pos.z);
                if !ft.sprung && g.state == "playing" && dx * dx + dz * dz < LURE_REACH * LURE_REACH {
                    g.story.false_trails[k].sprung = true;
                    g.story_lure(k);
                }
            });
        }
        self.story = StoryLevel {
            camp: Some(camp),
            walk: plan.walk,
            camp_step: plan.camp_step,
            trail,
            gaps: plan.gaps,
            false_trails,
            camp_come: false,
        };
        self.tape_glints();
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

impl Game {
    /// Her marks stay where she left them: settling the props round them
    /// (`settle_placement`) nudges a thing touching a wall out of it, and a
    /// torn corner against a wall or a mark on one is meant to touch it.
    pub fn story_marks_settled(&mut self) {
        let marks = self.story.trail.iter().chain(self.story.false_trails.iter().flat_map(|f| f.trail.iter()));
        for m in marks {
            if let Some(o) = &m.g {
                o.position().x = m.x;
                o.position().z = m.z;
                o.update_world_matrix(true, true);
            }
        }
    }

    /// Fit the story layer of a level just built to the run's progress: her
    /// page here, if it was already taken (a retry, a Continue), is gone.
    pub fn fit_story(&mut self) {
        let taken = self.story.camp.as_ref().and_then(|c| c.page_id).is_some_and(|id| self.notebook.contains(id));
        if let Some(c) = self.story.camp.as_mut().filter(|_| taken) {
            c.page_taken = true;
            if let Some(o) = &c.page {
                o.set_visible(false);
            }
            // and her lamp went out when you took it
            (c.lit, c.fade, c.dimming) = (false, 0.0, false);
            for o in [&c.flame, &c.light].into_iter().flatten() {
                o.set_visible(false);
            }
        }
        self.show_thread();
    }

    /// Take her page at the camp: it goes into the notebook (J), and its
    /// last line stays with you (`thread`).
    pub fn take_page(&mut self) {
        let Some(c) = self.story.camp.as_mut() else { return };
        let Some(id) = c.page_id.filter(|_| !c.page_taken) else { return };
        c.page_taken = true;
        if let Some(o) = &c.page {
            o.set_visible(false);
        }
        self.notebook.insert(id.to_string());
        if let Some(p) = notebook::page(id) {
            self.thread = p.thread.to_string();
        }
        // her lamp gutters out (`dress::camp_lamp`'s animator), with a breath
        // of air from where it stands, and the place wakes
        c.dimming = true;
        let at = c.lamp.as_ref().map(|l| {
            l.update_world_matrix(true, false);
            l.world_position()
        });
        self.show_thread();
        self.sfx_paper_rustle();
        self.sfx_page_flip();
        if let Some(p) = at {
            self.play_at(p.x, 0.3, p.z, 3.0, 1.0, |g| g.noise_burst(0.5, 320.0, 0.8, 0.12, "lowpass", Some(140.0), 0.25, 0.0));
        }
        // (the notebook's key as Settings bind it)
        let key = self.settings.keys(crate::session::input::Act::Journal).first().map_or_default(|k| crate::session::input::key_label(k));
        self.show_msg(&format!("Her page. {key} to read it."), 2.6);
        self.journal_newest = Some(crate::ui::hud::notebook_journal_id(self.level_num));
        self.director_note("story:page");
        self.director_wake();
        self.save_game();
    }

    /// The thread: the last line of the last page of hers you read, at the
    /// top of the screen (nothing before the first).
    pub fn show_thread(&mut self) {
        let t = if self.thread.is_empty() { String::new() } else { format!("\u{201c}{}\u{201d}", self.thread) };
        self.ui.objective.set_text(&t);
    }
}

impl Game {
    /// You followed a trail that is not hers to its end: something breathes
    /// close by in the dark ahead, and the place has you (`director_lure`).
    pub fn story_lure(&mut self, k: usize) {
        let Some(ft) = self.story.false_trails.get(k) else { return };
        let (ex, ez) = ft.end;
        let (dx, dz) = (ex - self.player.pos.x, ez - self.player.pos.z);
        let d = dx.dhypot(dz).max(0.5);
        self.play_at(ex + dx / d * 1.5, 1.2, ez + dz / d * 1.5, 3.0, 1.4, |g| g.sfx_breath());
        self.director_lure();
    }
}
