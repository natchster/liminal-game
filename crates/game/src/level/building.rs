//! building — the Building's floors (levels 1–3; `docs/MEGASTRUCTURE.md`).
//!
//! Pure module, like `archgen`: no scene, no engine objects, randomness only
//! from the injected `rng`, a fixed draw order. A floor of the Building is a
//! lattice: platforms (nodes, `PLAT`×`PLAT` tiles) every `PITCH` tiles,
//! joined by straight walks (edges: a gallery along a mass, a bridge over the
//! void, a flight of stairs) across the gaps between them; the squares
//! between the walks are building (a mass), void (a drop into the fog) or,
//! in the Works, more floor. A walk's ends meet their platforms at the
//! platform's height and walks touch one another only at a platform, so
//! heights change only along a flight or across a mass or a void
//! (`archgen::ground_breaks` is empty; `tests/building.rs` checks it).
//!
//! The walks are a spanning tree grown from the arrival (on from the newest
//! platform, a long winding way, and now and then from an older one, a
//! branch: a choice, a walk that goes nowhere), plus a few loops; the door
//! is the far point of it, against the outer wall, and the lowest floor;
//! heights fall from the arrival to the door along the way, with a step up
//! now and then. A lattice whose way is too straight, or has no choice on
//! it, too few flights or too few dead ends, is grown again
//! (`lattice_ok`). Cells (recesses with a hermetic door, the shelters from
//! the self-assembly) are cut into the masses, or into booths that stand
//! out over the void, so that every floor tile has one within `CELL_REACH`
//! tiles of walk; a floor where that cannot be (the arrival's corner left
//! with no building beside its walks) is laid again.

use crate::level::archgen::{
    build_ground_solids, ground_breaks, segment_clear_3d, Archetype, Ground, Layout, Room, Zone, DIRS, GROUND_FLOOR, GROUND_MASS,
    GROUND_STAIR, GROUND_VOID,
};

/// One storey, metres: a flight of one tile (4 m) rises one storey
pub const STOREY: f64 = 3.0;
/// Tiles from one platform to the next
pub const PITCH: i32 = 6;
/// A platform's side, tiles
pub const PLAT: i32 = 2;
/// The walk between two platforms, tiles
const GAP: i32 = PITCH - PLAT;
/// Where the void's fog bottoms out, and how high the masses rise, metres
pub const VOID_DEPTH: f32 = -100.0;
pub const MASS_TOP: f32 = 150.0;
/// Every floor tile has a cell within this many tiles of walk: as far as
/// you walk (`WALK`, 3.6 m/s) in 0.43 of the siren's warning
/// (`world::assembly::WARNING`, 21 s): 8 tiles, 32 m, 8.9 s. The first pass
/// had 16 tiles, 17.8 s of its 42; `tests/assembly.rs` holds the two
/// together
pub const CELL_REACH: i32 = 8;
/// How the floor falls along the way from the arrival (the share of the
/// way walked, to this power, is the share of the fall)
const FALL: f64 = 0.55;
/// The door's light: the lamp over it (`DOOR_LIGHT` over its floor), and
/// the windows of its stair tower lit above it, a storey apart, up to the
/// height of the arrival (`door_lights`): seen from across the void
pub const DOOR_LIGHT: f64 = 2.6;

/// The heights of the door's lights, metres: the lamp over the door, then
/// a lit window of its tower every storey up to `top` (the arrival's floor)
pub fn door_lights(door_floor: f64, top: f64) -> Vec<f64> {
    let mut v = vec![door_floor + DOOR_LIGHT];
    let mut y = door_floor + DOOR_LIGHT + STOREY;
    while y <= top + 2.0 * STOREY {
        v.push(y);
        y += STOREY;
    }
    v
}

/// The three places of the Building.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Style {
    /// galleries round a light well, the block opposite all windows (level 1)
    Well,
    /// a hall of piers and flights crossing at every height (level 2)
    Halls,
    /// terraces of dead machinery stepping down past shafts (level 3)
    Works,
}
impl Style {
    pub fn key(self) -> &'static str {
        match self {
            Style::Well => "well",
            Style::Halls => "halls",
            Style::Works => "works",
        }
    }
    /// The Building's floor on game level `level`, if it is one
    pub fn of_level(level: i32) -> Option<Style> {
        match level {
            1 => Some(Style::Well),
            2 => Some(Style::Halls),
            3 => Some(Style::Works),
            _ => None,
        }
    }
}

/// How many floors of the Building come before the rooms
pub const BUILDING_FLOORS: i32 = 3;
/// The first floor of the rooms: the Lobby
pub const FIRST_ROOMS_LEVEL: i32 = BUILDING_FLOORS + 1;

/// How far into the rooms game level `level` is: 1 on the Lobby (level 4),
/// 2 on the next, …; the Building's floors count as 1. What rots, darkens
/// and gets faster with depth counts this, not the level.
pub fn rooms_depth(level: i32) -> i32 {
    (level - BUILDING_FLOORS).max(1)
}

const fn bld(key: &'static str, title: &'static str, lore: &'static str, wall_mat: &'static str, floor_mat: &'static str) -> Archetype {
    Archetype {
        key,
        title,
        lore,
        ceil: [60.0, 60.0],
        wall: "solid",
        t: [4.0, 4.0],
        protrude: None,
        pillar: [4.0, 4.0],
        jamb: [0.0, 0.0],
        jamb_chance: 0.0,
        lintel: [2.4, 2.4],
        low_h: None,
        wall_mat,
        floor_mat,
        ceil_mat: "slab",
        light_every: 3,
        no_spawn: false,
    }
}

/// The Building's archetypes (looked up by `archgen::archetype` after the
/// rooms' own).
pub static BUILDING_ARCHETYPES: [Archetype; 3] = [
    bld("well", "THE WELL", "the Building", "panel", "deck"),
    bld("halls", "THE STAIR HALLS", "the Building", "ashlar", "stone"),
    bld("works", "THE WORKS", "the Building", "formwork", "factory"),
];

/// Platforms per side of a floor (a 28-tile floor, 112 m: the way from the
/// arrival to the door a median 85 tiles of walk; 7 a side was 40 tiles and
/// 165, `docs/MEGASTRUCTURE.md` G1)
const PLATFORMS: i32 = 5;
/// How often the tree grows on from an older platform than the newest (a
/// branch): at 0 a depth-first walk, which on a small lattice winds through
/// nearly every platform before the far one and leaves nothing off the way
const BRANCH: f64 = 0.15;
/// Lattices tried at most for one that keeps the bands (`lattice_ok`)
const TRIES: usize = 16;
/// Floors laid at most for one with a cell in reach of every tile
const FLOOR_TRIES: usize = 8;
/// The way to the door over the walks, at least this many times the
/// straight line (the tiles add a little: `tests/building.rs` holds them to 2)
const DETOUR: f64 = 2.2;

/// What a style asks of the lattice.
#[derive(Clone, Copy, Debug)]
struct Params {
    /// platforms per side
    n: i32,
    /// the arrival's storey, and how many storeys the floor falls to the door
    top: i32,
    range: i32,
    /// flight tiles per storey (2: a ramp)
    per_storey: i32,
    /// a lattice node is there
    node_p: f64,
    /// a walk not in the tree is laid anyway (a loop)
    loop_p: f64,
    /// a square is a mass / tries to be floor (the rest is void)
    mass_p: f64,
    floor_p: f64,
}
fn params(style: Style) -> Params {
    match style {
        Style::Well => Params { n: PLATFORMS, top: 5, range: 9, per_storey: 1, node_p: 0.9, loop_p: 0.18, mass_p: 0.0, floor_p: 0.0 },
        Style::Halls => Params { n: PLATFORMS, top: 6, range: 12, per_storey: 1, node_p: 0.85, loop_p: 0.28, mass_p: 0.3, floor_p: 0.0 },
        Style::Works => Params { n: PLATFORMS, top: 3, range: 6, per_storey: 2, node_p: 0.92, loop_p: 0.22, mass_p: 0.22, floor_p: 0.45 },
    }
}

/// A platform of the lattice.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Node {
    /// its top-left tile
    pub x: i32,
    pub z: i32,
    pub storey: i32,
    /// how many walks leave it
    pub degree: usize,
}

/// A shelter: a recess cut one tile into a mass, its hermetic door on the
/// side `door` (a `DIRS` index) toward the walk.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
    pub tile: (i32, i32),
    pub door: usize,
}

/// What the generator decided besides the tiles.
#[derive(Clone, Debug, PartialEq)]
pub struct BuildingPlan {
    pub style: Style,
    pub spawn: (i32, i32),
    /// the door's tile and the side of it the door is on (a `DIRS` index,
    /// toward the mass it is set in)
    pub exit: (i32, i32),
    pub exit_side: usize,
    pub cells: Vec<Cell>,
    pub nodes: Vec<Node>,
    /// the tiles the line from the arrival to the door's light crosses
    /// (nothing is built there after)
    pub view_line: Vec<(i32, i32)>,
}

/// A walk between two platforms.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Edge {
    a: usize,
    b: usize,
    /// across a missing platform (twice as long)
    long: bool,
    /// along x (else along z)
    along_x: bool,
}

#[inline]
fn irange(rng: &mut dyn FnMut() -> f64, a: i32, b: i32) -> i32 {
    a + (rng() * (b - a + 1) as f64).floor() as i32
}

/// The grid side for `n` platforms per side
pub fn side_for(n: i32) -> i32 {
    2 * PLAT + (n - 1) * PITCH
}

struct Grid {
    w: i32,
    g: Vec<Ground>,
}
impl Grid {
    fn idx(&self, x: i32, z: i32) -> usize {
        (z * self.w + x) as usize
    }
    fn get(&self, x: i32, z: i32) -> Ground {
        if x < 0 || z < 0 || x >= self.w || z >= self.w {
            return Ground::mass(VOID_DEPTH, MASS_TOP);
        }
        self.g[self.idx(x, z)]
    }
    fn set(&mut self, x: i32, z: i32, v: Ground) {
        let i = self.idx(x, z);
        self.g[i] = v;
    }
    fn fill(&mut self, x0: i32, z0: i32, x1: i32, z1: i32, v: Ground) {
        for z in z0..=z1 {
            for x in x0..=x1 {
                self.set(x, z, v);
            }
        }
    }
    fn walkable(&self, x: i32, z: i32) -> bool {
        self.get(x, z).walkable()
    }
}

/// Walk distances over a grid's walkable tiles from `from` (−1 unreachable)
fn grid_walk(grid: &Grid, from: &[(i32, i32)]) -> Vec<i32> {
    let w = grid.w;
    let mut d = vec![-1i32; (w * w) as usize];
    let mut q = std::collections::VecDeque::new();
    for &(x, z) in from {
        d[grid.idx(x, z)] = 0;
        q.push_back((x, z));
    }
    while let Some((x, z)) = q.pop_front() {
        let k = d[grid.idx(x, z)];
        for dd in DIRS {
            let (nx, nz) = (x + dd[0], z + dd[1]);
            if grid.walkable(nx, nz) && d[grid.idx(nx, nz)] < 0 {
                d[grid.idx(nx, nz)] = k + 1;
                q.push_back((nx, nz));
            }
        }
    }
    d
}

/// The platforms, the walks between them and their heights, before any
/// tile is laid (steps 1–5 of `generate_building`).
struct Lattice {
    /// the Well's well: the squares from `wa` to `wb` (exclusive) each way
    wa: i32,
    wb: i32,
    present: Vec<bool>,
    edges: Vec<Edge>,
    storey: Vec<i32>,
    exit_node: usize,
}

/// A lattice of platforms for `style`, drawing from `rng`.
fn lattice(style: Style, p: &Params, rng: &mut dyn FnMut() -> f64) -> Lattice {
    let n = p.n;
    let node_id = |i: i32, j: i32| (j * n + i) as usize;
    let nn = (n * n) as usize;

    // ---- 1. which platforms are there ----
    // (the Well's middle is its well, two or three squares across from the
    // arrival's side: only a few islands stand in it)
    let (wa, wb) = (1, irange(rng, n - 2, n - 1));
    let in_well = |i: i32, j: i32| style == Style::Well && i > wa && i < wb && j > wa && j < wb;
    let mut present = vec![false; nn];
    for j in 0..n {
        for i in 0..n {
            let r = rng();
            let pr = if in_well(i, j) { 0.3 } else { p.node_p };
            present[node_id(i, j)] = r < pr;
        }
    }
    let spawn_node = node_id(0, 0);
    present[spawn_node] = true;
    // the far corner and the far edges are always there: the door is on one
    for k in 0..n {
        if rng() < 0.75 {
            present[node_id(n - 1, k)] = true;
        }
        if rng() < 0.75 {
            present[node_id(k, n - 1)] = true;
        }
    }
    present[node_id(n - 1, n - 1)] = true;

    // ---- 2. the walks that could be laid ----
    let mut cand: Vec<Edge> = vec![];
    let mut crossed: Vec<usize> = vec![];
    for j in 0..n {
        for i in 0..n {
            let a = node_id(i, j);
            if !present[a] {
                continue;
            }
            for along_x in [true, false] {
                let step = |k: i32| if along_x { (i + k, j) } else { (i, j + k) };
                let (i1, j1) = step(1);
                if i1 >= n || j1 >= n {
                    continue;
                }
                if present[node_id(i1, j1)] {
                    cand.push(Edge { a, b: node_id(i1, j1), long: false, along_x });
                } else {
                    // across a missing platform, but only one walk crosses it
                    let (i2, j2) = step(2);
                    let mid = node_id(i1, j1);
                    if i2 < n && j2 < n && present[node_id(i2, j2)] && !crossed.contains(&mid) {
                        crossed.push(mid);
                        cand.push(Edge { a, b: node_id(i2, j2), long: true, along_x });
                    }
                }
            }
        }
    }

    // ---- 3. a spanning tree grown from the arrival: on from the newest
    // platform (a long way round), now and then from an older one (a branch:
    // a choice on the way, a walk that goes nowhere) ----
    let mut seen = vec![false; nn];
    seen[spawn_node] = true;
    let mut chosen = vec![false; cand.len()];
    let mut stack = vec![spawn_node];
    while !stack.is_empty() {
        let at =
            if stack.len() > 1 && rng() < BRANCH { ((rng() * stack.len() as f64) as usize).min(stack.len() - 1) } else { stack.len() - 1 };
        let cur = stack[at];
        let opts: Vec<usize> = (0..cand.len())
            .filter(|&k| {
                let e = cand[k];
                (e.a == cur && !seen[e.b]) || (e.b == cur && !seen[e.a])
            })
            .collect();
        if opts.is_empty() {
            stack.remove(at);
            continue;
        }
        let k = opts[((rng() * opts.len() as f64) as usize).min(opts.len() - 1)];
        chosen[k] = true;
        let other = if cand[k].a == cur { cand[k].b } else { cand[k].a };
        seen[other] = true;
        stack.push(other);
    }
    // platforms the walk never reached are not there
    for i in 0..nn {
        if !seen[i] {
            present[i] = false;
        }
    }
    // node distances over the chosen walks (and the order they are reached in)
    let graph_dist = |chosen: &[bool]| -> (Vec<i32>, Vec<usize>) {
        let mut dist = vec![-1i32; nn];
        dist[spawn_node] = 0;
        let mut q = std::collections::VecDeque::from([spawn_node]);
        let mut order = vec![];
        while let Some(a) = q.pop_front() {
            order.push(a);
            for (k, e) in cand.iter().enumerate() {
                if !chosen[k] {
                    continue;
                }
                let b = if e.a == a {
                    e.b
                } else if e.b == a {
                    e.a
                } else {
                    continue;
                };
                if dist[b] < 0 {
                    dist[b] = dist[a] + 1;
                    q.push_back(b);
                }
            }
        }
        (dist, order)
    };

    // ---- 4. the door: the far point of the tree, against the outer wall ----
    let (tree_dist, _) = graph_dist(&chosen);
    let on_outer = |a: usize| {
        let (i, j) = ((a as i32) % n, (a as i32) / n);
        i == n - 1 || j == n - 1
    };
    let exit_node = (0..nn)
        .filter(|&a| present[a] && tree_dist[a] > 0 && on_outer(a))
        .max_by_key(|&a| (tree_dist[a], (a as i32 % n) + (a as i32 / n)))
        .unwrap_or(node_id(n - 1, n - 1));

    // loops: a few walks beyond the tree, as long as the way to the door
    // stays long and some walks still go nowhere
    let leaves = |chosen: &[bool]| {
        (0..nn)
            .filter(|&a| a != spawn_node && a != exit_node && present[a])
            .filter(|&a| (0..cand.len()).filter(|&k| chosen[k] && (cand[k].a == a || cand[k].b == a)).count() == 1)
            .count()
    };
    let way = tree_dist[exit_node];
    for k in 0..cand.len() {
        let e = cand[k];
        if chosen[k] || !present[e.a] || !present[e.b] || rng() >= p.loop_p {
            continue;
        }
        chosen[k] = true;
        let (d, _) = graph_dist(&chosen);
        if (d[exit_node] as f64) < way as f64 * 0.8 || leaves(&chosen) < 2 {
            chosen[k] = false;
        }
    }
    let edges: Vec<Edge> = (0..cand.len()).filter(|&k| chosen[k]).map(|k| cand[k]).collect();
    let neighbours = |a: usize| -> Vec<usize> {
        edges
            .iter()
            .filter_map(|e| {
                if e.a == a {
                    Some(e.b)
                } else if e.b == a {
                    Some(e.a)
                } else {
                    None
                }
            })
            .collect()
    };
    let (dist, order) = graph_dist(&chosen);

    // ---- 5. heights: falling from the arrival to the door, a step up now and then ----
    let de = dist[exit_node].max(1) as f64;
    let mut storey = vec![0i32; nn];
    for &a in &order {
        let jitter = match rng() {
            r if r < 0.18 => 1,
            r if r < 0.36 => -1,
            _ => 0,
        };
        // (the floor falls fastest near the arrival: you arrive high over
        // the rest of it, and the line from there to the door stays above
        // the floors between)
        let fall = (dist[a] as f64 / de).min(1.0).powf(FALL);
        let s = p.top - (fall * p.range as f64).round() as i32 + if a == spawn_node || a == exit_node { 0 } else { jitter };
        storey[a] = s.clamp(p.top - p.range, p.top + 1);
    }
    // the Works stands in terraces: each block of 2×2 platforms at one height
    if style == Style::Works {
        let mut block_s: Vec<Option<i32>> = vec![None; nn];
        for &a in &order {
            let (i, j) = ((a as i32) % n, (a as i32) / n);
            let b = node_id(i / 2 * 2, j / 2 * 2);
            let s = *block_s[b].get_or_insert(storey[a]);
            if a != spawn_node && a != exit_node {
                storey[a] = s;
            }
        }
    }
    // a walk can only climb what its length allows: fit the tree in order, drop loops that cannot
    let cap = |e: &Edge| (if e.long { 2 * GAP + PLAT } else { GAP }) / p.per_storey;
    for &a in order.iter().skip(1) {
        let parent = neighbours(a).into_iter().filter(|&b| dist[b] == dist[a] - 1).min();
        if let Some(b) = parent {
            let e = edges.iter().find(|e| (e.a == a && e.b == b) || (e.a == b && e.b == a)).copied().unwrap();
            let c = cap(&e);
            storey[a] = storey[a].clamp(storey[b] - c, storey[b] + c);
        }
    }
    let lowest = (0..nn).filter(|&a| present[a]).map(|a| storey[a]).min().unwrap_or(0);
    // the door is the lowest floor (as low as its walks let it be)
    let mut s_exit = lowest;
    for b in neighbours(exit_node) {
        let e = edges.iter().find(|e| (e.a == exit_node && e.b == b) || (e.a == b && e.b == exit_node)).copied().unwrap();
        s_exit = s_exit.max(storey[b] - cap(&e));
    }
    storey[exit_node] = s_exit.min(storey[exit_node]);
    let mut edges: Vec<Edge> = edges.into_iter().filter(|e| (storey[e.a] - storey[e.b]).abs() <= cap(e)).collect();
    // what the dropped walks cut off is not there
    let mut reach = vec![false; nn];
    reach[spawn_node] = true;
    let mut q = vec![spawn_node];
    while let Some(a) = q.pop() {
        for e in &edges {
            let b = if e.a == a {
                e.b
            } else if e.b == a {
                e.a
            } else {
                continue;
            };
            if !reach[b] {
                reach[b] = true;
                q.push(b);
            }
        }
    }
    for a in 0..nn {
        present[a] = present[a] && reach[a];
    }
    edges.retain(|e| present[e.a] && present[e.b]);

    Lattice { wa, wb, present, edges, storey, exit_node }
}

/// Whether a lattice keeps the bands a floor is held to
/// (`tests/building.rs`): over its walks (a short one `PITCH` tiles, a long
/// one twice that), the way from the arrival to the door at least `DETOUR`
/// times the straight line, a platform where three or more walks meet on
/// it, three flights on it, and two walks that go nowhere.
fn lattice_ok(l: &Lattice, n: i32) -> bool {
    let nn = (n * n) as usize;
    let len = |e: &Edge| if e.long { 2 * PITCH } else { PITCH };
    let degree = |a: usize| l.edges.iter().filter(|e| e.a == a || e.b == a).count();
    // the shortest way over the walks (a few dozen platforms: relax until still)
    let mut d = vec![i32::MAX; nn];
    d[0] = 0;
    let mut changed = true;
    while changed {
        changed = false;
        for e in &l.edges {
            for (a, b) in [(e.a, e.b), (e.b, e.a)] {
                if d[a] != i32::MAX && d[a] + len(e) < d[b] {
                    d[b] = d[a] + len(e);
                    changed = true;
                }
            }
        }
    }
    let ex = l.exit_node;
    if d[ex] == i32::MAX {
        return false;
    }
    let (ie, je) = ((ex as i32) % n, (ex as i32) / n);
    let straight = ((ie * PITCH) as f64).hypot((je * PITCH) as f64).max(1.0);
    // back down the way from the door: its platforms and its flights
    let (mut cur, mut decisions, mut flights) = (ex, 0, 0);
    while cur != 0 {
        let Some(e) = l
            .edges
            .iter()
            .filter(|e| {
                (e.a == cur || e.b == cur) && {
                    let o = if e.a == cur { e.b } else { e.a };
                    d[o] != i32::MAX && d[o] + len(e) == d[cur]
                }
            })
            .min_by_key(|e| if e.a == cur { e.b } else { e.a })
        else {
            return false;
        };
        let o = if e.a == cur { e.b } else { e.a };
        flights += usize::from(l.storey[e.a] != l.storey[e.b]);
        if o != 0 && degree(o) >= 3 {
            decisions += 1;
        }
        cur = o;
    }
    let dead = (1..nn).filter(|&a| a != ex && l.present[a] && degree(a) == 1).count();
    d[ex] as f64 >= DETOUR * straight && decisions >= 1 && flights >= 3 && dead >= 2
}

/// Generates the Building's floor for `level` (one of `BUILDING_FLOORS`),
/// drawing from `rng` (the game passes `&mut || rnd()`).
/// A lattice of platforms for `style` that keeps the bands (a long way
/// round, a choice on it, flights, dead ends; `lattice_ok`), tried up to
/// `TRIES` times (steps 1–5 of `generate_building`).
fn good_lattice(style: Style, p: &Params, rng: &mut dyn FnMut() -> f64) -> Lattice {
    let mut lat = lattice(style, p, rng);
    for _ in 1..TRIES {
        if lattice_ok(&lat, p.n) {
            break;
        }
        lat = lattice(style, p, rng);
    }
    lat
}

/// A floor laid on a lattice (steps 6–11 of `generate_building`).
struct Floor {
    grid: Grid,
    exit: (i32, i32),
    exit_side: usize,
    /// the mass the door is set in
    door_wall: (i32, i32),
    cells: Vec<Cell>,
    nodes: Vec<Node>,
    /// the tiles of the line from the arrival to the door's light
    kept: Vec<(i32, i32)>,
    /// every floor tile has a cell within `CELL_REACH` tiles of walk
    served: bool,
}

/// The squares, the platforms and walks, the door, the line to its light
/// and the cells, on lattice `lat`.
fn lay_floor(style: Style, p: &Params, lat: Lattice, rng: &mut dyn FnMut() -> f64) -> Floor {
    let n = p.n;
    let w = side_for(n);
    let node_id = |i: i32, j: i32| (j * n + i) as usize;
    let node_xy = |i: i32, j: i32| (PLAT / 2 + i * PITCH, PLAT / 2 + j * PITCH);
    let Lattice { wa, wb, present, edges, storey, exit_node } = lat;
    let nn = (n * n) as usize;

    // ---- 6. the squares between the walks: building, void, or more floor ----
    let mut grid = Grid { w, g: vec![Ground::mass(VOID_DEPTH, MASS_TOP); (w * w) as usize] };
    let nsq = n - 1;
    // 0 void, 1 mass, 2 floor (tried)
    let mut sq = vec![0u8; (nsq * nsq) as usize];
    for j in 0..nsq {
        for i in 0..nsq {
            let r = rng();
            sq[(j * nsq + i) as usize] = match style {
                Style::Well => {
                    let well = i >= wa && i < wb && j >= wa && j < wb;
                    u8::from(!well && r >= 0.12)
                }
                _ if r < p.mass_p => 1,
                _ if r < p.mass_p + p.floor_p => 2,
                _ => 0,
            };
        }
    }
    // everything inside the border starts as void; the squares that are building stand in it
    grid.fill(1, 1, w - 2, w - 2, Ground::void());
    let sq_rect = |i: i32, j: i32| {
        let (x, z) = node_xy(i, j);
        (x + PLAT, z + PLAT, x + PITCH - 1, z + PITCH - 1)
    };
    for j in 0..nsq {
        for i in 0..nsq {
            if sq[(j * nsq + i) as usize] == 1 {
                let (x0, z0, x1, z1) = sq_rect(i, j);
                grid.fill(x0, z0, x1, z1, Ground::mass(VOID_DEPTH, MASS_TOP));
            }
        }
    }
    // a gap with no walk in it, between two masses, is building too
    for j in 0..n {
        for i in 0..n {
            let (x, z) = node_xy(i, j);
            let sq_at = |a: i32, b: i32| if a >= 0 && b >= 0 && a < nsq && b < nsq { sq[(b * nsq + a) as usize] } else { 1 };
            if i < n - 1 && sq_at(i, j - 1) == 1 && sq_at(i, j) == 1 {
                grid.fill(x + PLAT, z, x + PITCH - 1, z + PLAT - 1, Ground::mass(VOID_DEPTH, MASS_TOP));
            }
            if j < n - 1 && sq_at(i - 1, j) == 1 && sq_at(i, j) == 1 {
                grid.fill(x, z + PLAT, x + PLAT - 1, z + PITCH - 1, Ground::mass(VOID_DEPTH, MASS_TOP));
            }
        }
    }

    // ---- 7. platforms and walks ----
    let h_of = |s: i32| (s as f64 * STOREY) as f32;
    let mut nodes_out = vec![];
    for a in 0..nn {
        if !present[a] {
            continue;
        }
        let (i, j) = ((a as i32) % n, (a as i32) / n);
        let (x, z) = node_xy(i, j);
        grid.fill(x, z, x + PLAT - 1, z + PLAT - 1, Ground::floor(h_of(storey[a])));
        nodes_out.push(Node { x, z, storey: storey[a], degree: edges.iter().filter(|e| e.a == a || e.b == a).count() });
    }
    for e in &edges {
        let (ia, ja) = ((e.a as i32) % n, (e.a as i32) / n);
        let (xa, za) = node_xy(ia, ja);
        let len = if e.long { 2 * GAP + PLAT } else { GAP };
        let (sa, sb) = (storey[e.a], storey[e.b]);
        let k = (sa - sb).abs() * p.per_storey;
        let off = irange(rng, 0, len - k);
        // which row of the platforms it runs along; now and then both (a wide walk)
        let row = irange(rng, 0, PLAT - 1);
        let wide = style != Style::Halls && rng() < 0.25;
        let dir = if e.along_x { 0u8 } else { 2u8 };
        let step = STOREY / p.per_storey as f64 * if sb < sa { -1.0 } else { 1.0 };
        for t in 0..len {
            let g = if t < off {
                Ground::floor(h_of(sa))
            } else if t < off + k {
                let h0 = sa as f64 * STOREY + step * (t - off) as f64;
                Ground::stair(h0 as f32, (h0 + step) as f32, dir)
            } else {
                Ground::floor(h_of(sb))
            };
            for r in 0..PLAT {
                if r != row && !wide {
                    continue;
                }
                let (x, z) = if e.along_x { (xa + PLAT + t, za + r) } else { (xa + r, za + PLAT + t) };
                grid.set(x, z, g);
            }
        }
    }

    // ---- 8. the Works' terraces: a square of floor where it meets its walks level ----
    if style == Style::Works {
        let (xe, ze) = node_xy((exit_node as i32) % n, (exit_node as i32) / n);
        let way_of = |grid: &Grid| grid_walk(grid, &[(1, 1)])[grid.idx(xe, ze)];
        let way0 = way_of(&grid);
        for j in 0..nsq {
            for i in 0..nsq {
                if sq[(j * nsq + i) as usize] != 2 {
                    continue;
                }
                let corners = [node_id(i, j), node_id(i + 1, j), node_id(i, j + 1), node_id(i + 1, j + 1)];
                let s = storey[corners[0]];
                if !corners.iter().all(|&c| present[c] && storey[c] == s) {
                    continue;
                }
                let (x0, z0, x1, z1) = sq_rect(i, j);
                let before: Vec<Ground> = grid.g.clone();
                grid.fill(x0, z0, x1, z1, Ground::floor(h_of(s)));
                // the walks' other rows round it become floor too where they were void
                for z in (z0 - PLAT).max(1)..=(z1 + PLAT).min(w - 2) {
                    for x in (x0 - PLAT).max(1)..=(x1 + PLAT).min(w - 2) {
                        if grid.get(x, z).kind == GROUND_VOID {
                            grid.set(x, z, Ground::floor(h_of(s)));
                        }
                    }
                }
                if !ground_breaks(w, w, &grid.g).is_empty() || (way_of(&grid) as f64) < way0 as f64 * 0.85 {
                    grid.g = before;
                }
            }
        }
    }

    // ---- 9. the door: a platform tile of its node against the outer wall ----
    let (ie, je) = ((exit_node as i32) % n, (exit_node as i32) / n);
    let (xe, ze) = node_xy(ie, je);
    let mut exit = (xe + PLAT - 1, ze + PLAT - 1);
    let mut exit_side = 0usize;
    'door: for dz in (0..PLAT).rev() {
        for dx in (0..PLAT).rev() {
            for side in [0usize, 2, 1, 3] {
                let (x, z) = (xe + dx, ze + dz);
                let (mx, mz) = (x + DIRS[side][0], z + DIRS[side][1]);
                if grid.get(mx, mz).kind == GROUND_MASS {
                    exit = (x, z);
                    exit_side = side;
                    break 'door;
                }
            }
        }
    }

    // ---- 10. the door seen from the arrival: building on the line between is cut away ----
    // (from the arrival's platform, the eye to the door's sign; the line
    // must not pass through a walk's slab, and the border and the door's
    // own wall stay)
    let door_wall = (exit.0 + DIRS[exit_side][0], exit.1 + DIRS[exit_side][1]);
    let mut kept: Vec<(i32, i32)> = vec![];
    {
        let floor_h = |g: &Ground, x: i32, z: i32| g.at(x, z, x as f64 + 0.5, z as f64 + 0.5);
        let lights = door_lights(floor_h(&grid.get(exit.0, exit.1), exit.0, exit.1), p.top as f64 * STOREY);
        // the tiles a line crosses, with the heights it has where it enters and leaves each
        let crossed = |from: (f64, f64, f64), to: (f64, f64, f64)| -> Vec<((i32, i32), f64, f64)> {
            let steps = 400;
            let mut out: Vec<((i32, i32), f64, f64)> = vec![];
            for k in 0..=steps {
                let t = k as f64 / steps as f64;
                let (u, v, y) = (from.0 + (to.0 - from.0) * t, from.1 + (to.1 - from.1) * t, from.2 + (to.2 - from.2) * t);
                let tile = (u.floor() as i32, v.floor() as i32);
                match out.last_mut() {
                    Some(last) if last.0 == tile => last.2 = y,
                    _ => out.push((tile, y, y)),
                }
            }
            out
        };
        let mut best: Option<(usize, Vec<(i32, i32)>, Vec<(i32, i32)>)> = None;
        let near_spawn = grid_walk(&grid, &[(1, 1)]);
        for z in 1..w - 1 {
            for x in 1..w - 1 {
                if !(0..=3).contains(&near_spawn[grid.idx(x, z)]) {
                    continue;
                }
                let from = (x as f64 + 0.5, z as f64 + 0.5, floor_h(&grid.get(x, z), x, z) + 1.6);
                for &ly in &lights {
                    let target = (exit.0 as f64 + 0.5, exit.1 as f64 + 0.5, ly);
                    let mut cut = vec![];
                    let mut line = vec![];
                    let mut ok = true;
                    for (tile, ya, yb) in crossed(from, target) {
                        line.push(tile);
                        if tile == (x, z) || tile == exit {
                            continue;
                        }
                        let g = grid.get(tile.0, tile.1);
                        match g.kind {
                            GROUND_MASS => {
                                let border = tile.0 < 1 || tile.1 < 1 || tile.0 > w - 2 || tile.1 > w - 2;
                                if border || tile == door_wall {
                                    ok = false;
                                } else {
                                    cut.push(tile);
                                }
                            }
                            GROUND_VOID => {}
                            _ => {
                                // a walk under the line is fine, through its slab is not
                                let top = g.h0.min(g.h1) as f64;
                                if ya.min(yb) <= top + 0.1 && ya.max(yb) >= top - 0.5 {
                                    ok = false;
                                }
                            }
                        }
                    }
                    if ok && best.as_ref().is_none_or(|b| cut.len() < b.0) {
                        best = Some((cut.len(), cut, line));
                    }
                }
            }
        }
        if let Some((_, cut, line)) = best {
            for t in cut {
                grid.set(t.0, t.1, Ground::void());
            }
            // nothing is built on the line after (booths, cells)
            kept = line;
        }
    }

    // ---- 11. cells: every floor tile within `CELL_REACH` tiles of walk of one ----
    // (the floor farthest from a cell is served each time, the arrival
    // first, by the cell in reach of it that brings the most floor in reach:
    // cut into the building beside a walk where one can be, else cut into a
    // booth that stands out over the void beside the walk)
    // a cell: a mass tile (not on the border) with one floor neighbour, and building on its other three sides
    let cell_at = |grid: &Grid, cells: &[Cell], x: i32, z: i32| -> Option<usize> {
        if x < 1 || z < 1 || x > w - 2 || z > w - 2 || grid.get(x, z).kind != GROUND_MASS || (x, z) == door_wall {
            return None;
        }
        let mut door = None;
        for (d, dd) in DIRS.iter().enumerate() {
            let (nx, nz) = (x + dd[0], z + dd[1]);
            let g = grid.get(nx, nz);
            if g.kind == GROUND_FLOOR && door.is_none() && (nx, nz) != exit && !cells.iter().any(|c| c.tile == (nx, nz)) {
                door = Some(d);
            } else if g.kind != GROUND_MASS {
                return None;
            }
        }
        door
    };
    let in_reach = |v: i32| (0..=CELL_REACH).contains(&v);
    // the floor a cell at `tile` would bring in reach that the walk from the
    // cells there are (`d`) does not
    let gain = |grid: &Grid, d: &[i32], tile: (i32, i32)| -> usize {
        let dc = grid_walk(grid, &[tile]);
        (0..grid.g.len()).filter(|&i| grid.g[i].walkable() && in_reach(dc[i]) && !in_reach(d[i])).count()
    };
    let mut cells: Vec<Cell> = vec![];
    // floor no cell could be cut near (left out of the search after)
    let mut unserved: Vec<(i32, i32)> = vec![];
    for _ in 0..64 {
        let from: Vec<(i32, i32)> = cells.iter().map(|c| c.tile).collect();
        let d = if from.is_empty() { vec![-1; (w * w) as usize] } else { grid_walk(&grid, &from) };
        // the floor tile farthest from a cell (the arrival first)
        let mut far: Option<((i32, i32), i32)> = None;
        if from.is_empty() && !unserved.contains(&(1, 1)) {
            far = Some(((1, 1), i32::MAX));
        } else {
            for z in 0..w {
                for x in 0..w {
                    let v = d[grid.idx(x, z)];
                    let v = if v < 0 { i32::MAX } else { v };
                    if grid.walkable(x, z) && v > CELL_REACH && !unserved.contains(&(x, z)) && far.is_none_or(|f| v > f.1) {
                        far = Some(((x, z), v));
                    }
                }
            }
        }
        let Some((f, _)) = far else { break };
        let df = grid_walk(&grid, &[f]);
        // cut into the building beside a walk: of the places in reach of it,
        // the one that brings the most floor in reach (the nearer of two)
        let mut best: Option<(Cell, usize, i32)> = None;
        for z in 1..w - 1 {
            for x in 1..w - 1 {
                let Some(door) = cell_at(&grid, &cells, x, z) else { continue };
                // (the cell is a step past its doorway)
                let v = df[grid.idx(x + DIRS[door][0], z + DIRS[door][1])];
                if v < 0 || v + 1 > CELL_REACH {
                    continue;
                }
                let n = gain(&grid, &d, (x, z));
                if best.as_ref().is_none_or(|b| n > b.1 || (n == b.1 && v < b.2)) {
                    best = Some((Cell { tile: (x, z), door }, n, v));
                }
            }
        }
        // none: a booth of building (3 × 2 tiles) stands out over the void
        // beside the walk, and the cell is cut into it: off the line to the
        // door (a floor where only that would do is laid again), and over the
        // void rather than on a walk
        if best.is_none() {
            let reach0 = grid_walk(&grid, &[(1, 1)]).iter().filter(|&&v| v >= 0).count();
            // (the cell, the floor it brings in reach, the walk it takes, its distance, the floor with it)
            let mut booth: Option<(Cell, usize, usize, i32, Vec<Ground>)> = None;
            for z in 1..w - 1 {
                for x in 1..w - 1 {
                    let v = df[grid.idx(x, z)];
                    if !(0..CELL_REACH).contains(&v) || grid.get(x, z).kind != GROUND_FLOOR || cells.iter().any(|c| c.tile == (x, z)) {
                        continue;
                    }
                    for (dir, dd) in DIRS.iter().enumerate() {
                        let (px, pz) = if dd[0] != 0 { (0, 1) } else { (1, 0) };
                        let block: Vec<(i32, i32)> = [0, 1]
                            .iter()
                            .flat_map(|&k| [-1, 0, 1].map(move |s| (x + dd[0] * (1 + k) + px * s, z + dd[1] * (1 + k) + pz * s)))
                            .collect();
                        let spare = |bx: i32, bz: i32| {
                            let g = grid.get(bx, bz);
                            (g.kind == GROUND_VOID || g.kind == GROUND_FLOOR)
                                && bx >= 1
                                && bz >= 1
                                && bx <= w - 2
                                && bz <= w - 2
                                && (bx, bz) != (1, 1)
                                && (bx, bz) != exit
                                && !kept.contains(&(bx, bz))
                                && !cells
                                    .iter()
                                    .any(|c| c.tile == (bx, bz) || (c.tile.0 + DIRS[c.door][0], c.tile.1 + DIRS[c.door][1]) == (bx, bz))
                        };
                        if !block.iter().all(|&(bx, bz)| spare(bx, bz)) {
                            continue;
                        }
                        let lost = block.iter().filter(|&&(bx, bz)| grid.walkable(bx, bz)).count();
                        let mut g2 = Grid { w, g: grid.g.clone() };
                        for &(bx, bz) in &block {
                            g2.set(bx, bz, Ground::mass(VOID_DEPTH, MASS_TOP));
                        }
                        // nothing may be cut off, and the floor it serves stays in reach
                        let reach1 = grid_walk(&g2, &[(1, 1)]);
                        let all = (0..g2.g.len()).all(|i| !g2.g[i].walkable() || reach1[i] >= 0);
                        if !all || reach1.iter().filter(|&&v| v >= 0).count() + lost != reach0 {
                            continue;
                        }
                        let cell = (x + dd[0], z + dd[1]);
                        if !in_reach(grid_walk(&g2, &[cell])[g2.idx(f.0, f.1)]) {
                            continue;
                        }
                        let n = gain(&g2, &d, cell);
                        if booth.as_ref().is_none_or(|b| (lost, std::cmp::Reverse(n), v) < (b.2, std::cmp::Reverse(b.1), b.3)) {
                            booth = Some((Cell { tile: cell, door: dir ^ 1 }, n, lost, v, g2.g));
                        }
                    }
                }
            }
            if let Some((c, n, _, v, g)) = booth {
                grid.g = g;
                best = Some((c, n, v));
            }
        }
        let Some((c, _, _)) = best else {
            unserved.push(f);
            continue;
        };
        let (dx, dz) = (c.tile.0 + DIRS[c.door][0], c.tile.1 + DIRS[c.door][1]);
        let h = grid.get(dx, dz).h0;
        grid.set(c.tile.0, c.tile.1, Ground::floor(h));
        cells.push(c);
    }
    let reach = grid_walk(&grid, &cells.iter().map(|c| c.tile).collect::<Vec<_>>());
    let served = (0..grid.g.len()).all(|i| !grid.g[i].walkable() || (0..=CELL_REACH).contains(&reach[i]));
    Floor { grid, exit, exit_side, door_wall, cells, nodes: nodes_out, kept, served }
}

pub fn generate_building(level: i32, rng: &mut dyn FnMut() -> f64) -> (Layout, BuildingPlan) {
    let style = Style::of_level(level).unwrap_or(Style::Well);
    let p = params(style);
    let w = side_for(p.n);

    // ---- 1–11. the platforms, the walks, their heights (a lattice that
    // keeps the bands), the floor laid on them, the door and the cells: laid
    // again on a new lattice while some floor has no cell in reach (where
    // the arrival's corner leaves no building to cut one into) ----
    let mut floor = lay_floor(style, &p, good_lattice(style, &p, rng), rng);
    for _ in 1..FLOOR_TRIES {
        if floor.served {
            break;
        }
        floor = lay_floor(style, &p, good_lattice(style, &p, rng), rng);
    }
    let Floor { mut grid, exit, exit_side, door_wall, cells, nodes: nodes_out, kept, .. } = floor;

    // ---- 12. the Works' masses are machine housings: each block of them
    // stands 4–9 m over the highest floor beside it (the outer wall and the
    // door's wall rise out of sight) ----
    if style == Style::Works {
        let mut seen = vec![false; (w * w) as usize];
        for z0 in 1..w - 1 {
            for x0 in 1..w - 1 {
                if seen[grid.idx(x0, z0)] || grid.get(x0, z0).kind != GROUND_MASS {
                    continue;
                }
                // the block: masses joined side by side, inside the border
                let mut block = vec![(x0, z0)];
                seen[grid.idx(x0, z0)] = true;
                let mut k = 0;
                while k < block.len() {
                    let (x, z) = block[k];
                    k += 1;
                    for dd in DIRS {
                        let (nx, nz) = (x + dd[0], z + dd[1]);
                        if nx >= 1
                            && nz >= 1
                            && nx <= w - 2
                            && nz <= w - 2
                            && !seen[grid.idx(nx, nz)]
                            && grid.get(nx, nz).kind == GROUND_MASS
                        {
                            seen[grid.idx(nx, nz)] = true;
                            block.push((nx, nz));
                        }
                    }
                }
                let border = block.iter().any(|&(x, z)| x == 1 || z == 1 || x == w - 2 || z == w - 2);
                let near = block
                    .iter()
                    .flat_map(|&(x, z)| DIRS.iter().map(move |dd| (x + dd[0], z + dd[1])))
                    .filter(|&(x, z)| grid.walkable(x, z));
                let top = near.map(|(x, z)| grid.get(x, z).h0.max(grid.get(x, z).h1)).fold(f32::NEG_INFINITY, f32::max);
                if border || block.contains(&door_wall) || !top.is_finite() {
                    continue;
                }
                let top = top + (4.0 + rng() * 5.0) as f32;
                for &(x, z) in &block {
                    grid.set(x, z, Ground::mass(VOID_DEPTH, top));
                }
            }
        }
    }

    // ---- 13. the layout: tiles, districts as zones ----
    let tiles: Vec<u8> = grid.g.iter().map(|g| u8::from(!g.walkable())).collect();
    let arch = BUILDING_ARCHETYPES.iter().find(|a| a.key == style.key()).unwrap();
    let mut zones = vec![];
    let cuts = [0, w / 2, w];
    for zj in 0..2 {
        for zi in 0..2 {
            let (x0, z0, x1, z1) = (cuts[zi].max(1), cuts[zj].max(1), (cuts[zi + 1] - 1).min(w - 2), (cuts[zj + 1] - 1).min(w - 2));
            zones.push(Zone {
                id: zones.len(),
                arch: arch.key,
                x0,
                z0,
                x1,
                z1,
                title: arch.title,
                lore: arch.lore,
                ceil_h: arch.ceil[0],
                wall: arch.wall,
                wall_t: arch.t[0],
                pillar: arch.pillar[0],
                protrude: 0.0,
                jamb: 0.0,
                lintel_h: arch.lintel[0],
                low_h: 0.0,
                wall_mat: arch.wall_mat,
                floor_mat: arch.floor_mat,
                ceil_mat: arch.ceil_mat,
                light_every: arch.light_every,
            });
        }
    }
    let mut zone_of = vec![0u8; (w * w) as usize];
    for z in 0..w {
        for x in 0..w {
            let zi = usize::from(x >= w / 2) + 2 * usize::from(z >= w / 2);
            zone_of[(z * w + x) as usize] = zi as u8;
        }
    }
    let rooms = zones.iter().map(|z| Room { zone: z.id, x0: z.x0, z0: z.z0, x1: z.x1, z1: z.z1 }).collect();
    let layout = Layout {
        w,
        h: w,
        cell: 4.0,
        tiles,
        zone_of,
        tile_flags: vec![0u8; (w * w) as usize],
        zones,
        dominant: arch.key,
        level,
        scp: None,
        rooms,
        ground: grid.g,
    };
    let plan = BuildingPlan { style, spawn: (1, 1), exit, exit_side, cells, nodes: nodes_out, view_line: kept };
    (layout, plan)
}

/// How hard a floor is to find the way down through (`docs/MEGASTRUCTURE.md`,
/// B2): measured on the tiles, as the player walks them.
#[derive(Clone, Debug, PartialEq)]
pub struct Navigation {
    /// walkable tiles
    pub walkable: usize,
    /// tiles of walk from the arrival to the door, and the straight line
    pub path: i32,
    pub straight: f64,
    /// `path / straight`
    pub detour: f64,
    /// flights of stairs on the way, and metres climbed and descended on it
    pub flights: usize,
    pub climb: f64,
    pub descent: f64,
    /// the floor's heights, metres
    pub low: f64,
    pub high: f64,
    /// platforms on the way where three or more walks meet
    pub decisions: usize,
    /// platforms only one walk reaches (not the arrival, not the door)
    pub dead_ends: usize,
    /// the door's light seen from a tile near the arrival
    pub door_seen: bool,
    /// tiles of walk from the farthest floor to a cell
    pub cell_reach: i32,
    pub cells: usize,
}

/// A cell's front: the wall across its open side, with the doorway in its
/// middle (`CELL_DOOR` wide, `CELL_DOOR_H` high), from the floor to the top
/// of the building over it. Boxes as `build_ground_solids` takes them.
pub const CELL_DOOR: f64 = 1.3;
pub const CELL_DOOR_H: f64 = 2.1;
/// The front wall's thickness, metres
pub const CELL_FRONT: f64 = 0.3;
pub fn cell_front_boxes(p: &BuildingPlan, ground: &[Ground], w: i32, cell: f64) -> Vec<[f32; 8]> {
    let mut out = vec![];
    for c in &p.cells {
        let (x, z) = c.tile;
        let g = ground[(z * w + x) as usize];
        // (as high as the building it is cut into)
        let top = DIRS
            .iter()
            .filter_map(|dd| ground.get(((z + dd[1]) * w + x + dd[0]) as usize))
            .filter(|n| n.kind == GROUND_MASS)
            .map(|n| n.h1)
            .fold(g.h0 + 3.0, f32::max);
        let h = g.h0;
        let t = (z * w + x) as f32;
        let (th, half) = ((CELL_FRONT / cell) as f32, (CELL_DOOR / cell / 2.0) as f32);
        let d = DIRS[c.door];
        // the wall's band across the tile, on the door's side
        let (a0, a1) = if d[0] + d[1] > 0 { (1.0 - th, 1.0) } else { (0.0, th) };
        let mut push = |s0: f32, s1: f32, y0: f32, y1: f32| {
            let b = if d[0] != 0 {
                [x as f32 + a0, z as f32 + s0, x as f32 + a1, z as f32 + s1]
            } else {
                [x as f32 + s0, z as f32 + a0, x as f32 + s1, z as f32 + a1]
            };
            out.push([b[0], b[1], b[2], b[3], y0, y1, t, crate::level::archgen::BOX_WALL as f32]);
        };
        push(0.0, 0.5 - half, h, top);
        push(0.5 + half, 1.0, h, top);
        push(0.5 - half, 0.5 + half, h + CELL_DOOR_H as f32, top);
    }
    out
}

/// The floor under tile (x, z)'s centre, metres
fn centre_h(l: &Layout, x: i32, z: i32) -> f64 {
    let g = l.ground[(z * l.w + x) as usize];
    g.at(x, z, x as f64 + 0.5, z as f64 + 0.5)
}

/// Walk distances over the open tiles from `from` (−1 unreachable)
pub fn walk_from(l: &Layout, from: &[(i32, i32)]) -> Vec<i32> {
    let mut d = vec![-1i32; (l.w * l.h) as usize];
    let mut q = std::collections::VecDeque::new();
    for &(x, z) in from {
        d[(z * l.w + x) as usize] = 0;
        q.push_back((x, z));
    }
    while let Some((x, z)) = q.pop_front() {
        let k = d[(z * l.w + x) as usize];
        for dd in DIRS {
            let (nx, nz) = (x + dd[0], z + dd[1]);
            if nx < 0 || nz < 0 || nx >= l.w || nz >= l.h {
                continue;
            }
            let i = (nz * l.w + nx) as usize;
            if l.tiles[i] == 0 && d[i] < 0 {
                d[i] = k + 1;
                q.push_back((nx, nz));
            }
        }
    }
    d
}

/// The navigation measures of a floor.
pub fn navigation(l: &Layout, p: &BuildingPlan) -> Navigation {
    let idx = |x: i32, z: i32| (z * l.w + x) as usize;
    let walkable = l.tiles.iter().filter(|&&t| t == 0).count();
    let ds = walk_from(l, &[p.spawn]);
    let de = walk_from(l, &[p.exit]);
    let path = ds[idx(p.exit.0, p.exit.1)];
    let straight = (((p.exit.0 - p.spawn.0).pow(2) + (p.exit.1 - p.spawn.1).pow(2)) as f64).sqrt();
    // the way, down the door's distances from the arrival
    let mut way = vec![p.spawn];
    let mut cur = p.spawn;
    while de[idx(cur.0, cur.1)] > 0 {
        let k = de[idx(cur.0, cur.1)];
        let Some(nx) = DIRS
            .iter()
            .map(|d| (cur.0 + d[0], cur.1 + d[1]))
            .find(|&(x, z)| x >= 0 && z >= 0 && x < l.w && z < l.h && de[idx(x, z)] == k - 1)
        else {
            break;
        };
        way.push(nx);
        cur = nx;
    }
    let (mut flights, mut climb, mut descent) = (0usize, 0.0, 0.0);
    let mut on_stair = false;
    for k in 1..way.len() {
        let (a, b) = (way[k - 1], way[k]);
        let dh = centre_h(l, b.0, b.1) - centre_h(l, a.0, a.1);
        if dh > 0.0 {
            climb += dh;
        } else {
            descent -= dh;
        }
        let stair = l.ground[idx(b.0, b.1)].kind == GROUND_STAIR;
        if stair && !on_stair {
            flights += 1;
        }
        on_stair = stair;
    }
    let (mut low, mut high) = (f64::INFINITY, f64::NEG_INFINITY);
    for z in 0..l.h {
        for x in 0..l.w {
            if l.tiles[idx(x, z)] == 0 {
                let h = centre_h(l, x, z);
                low = low.min(h);
                high = high.max(h);
            }
        }
    }
    let on_way = |nd: &Node| way.iter().any(|&(x, z)| x >= nd.x && x < nd.x + PLAT && z >= nd.z && z < nd.z + PLAT);
    let decisions = p.nodes.iter().filter(|nd| nd.degree >= 3 && on_way(nd)).count();
    let is_end = |nd: &Node| {
        (p.spawn.0 >= nd.x && p.spawn.0 < nd.x + PLAT && p.spawn.1 >= nd.z && p.spawn.1 < nd.z + PLAT)
            || (p.exit.0 >= nd.x && p.exit.0 < nd.x + PLAT && p.exit.1 >= nd.z && p.exit.1 < nd.z + PLAT)
    };
    let dead_ends = p.nodes.iter().filter(|nd| nd.degree == 1 && !is_end(nd)).count();
    // the door's light, seen from near the arrival (eye to the sign)
    let solids = build_ground_solids(l.w, l.h, l.cell, &l.ground);
    let (ex, ez) = (p.exit.0 as f64 + 0.5, p.exit.1 as f64 + 0.5);
    let high_floor = l.ground.iter().filter(|g| g.walkable()).map(|g| g.h0.max(g.h1) as f64).fold(f64::NEG_INFINITY, f64::max);
    let lights = door_lights(centre_h(l, p.exit.0, p.exit.1), centre_h(l, p.spawn.0, p.spawn.1).min(high_floor));
    let mut door_seen = false;
    'seen: for z in 0..l.h {
        for x in 0..l.w {
            let d = ds[idx(x, z)];
            if (0..=3).contains(&d) {
                let y = centre_h(l, x, z) + 1.6;
                if lights.iter().any(|&ey| segment_clear_3d(&solids, x as f64 + 0.5, z as f64 + 0.5, y, ex, ez, ey)) {
                    door_seen = true;
                    break 'seen;
                }
            }
        }
    }
    let dc = walk_from(l, &p.cells.iter().map(|c| c.tile).collect::<Vec<_>>());
    let cell_reach = (0..l.tiles.len()).filter(|&i| l.tiles[i] == 0).map(|i| if dc[i] < 0 { i32::MAX } else { dc[i] }).max().unwrap_or(0);
    Navigation {
        walkable,
        path,
        straight,
        detour: if straight > 0.0 { path as f64 / straight } else { 0.0 },
        flights,
        climb,
        descent,
        low,
        high,
        decisions,
        dead_ends,
        door_seen,
        cell_reach,
        cells: p.cells.len(),
    }
}

/// The floor as text, a row per line: `#` mass, ` ` void, a digit or letter
/// for the storey of a floor (`0`–`9` from the lowest, then `a`…), `>` `<`
/// `v` `^` for a flight (the way it goes down), `C` a cell, `S` the
/// arrival, `E` the door.
pub fn ascii(l: &Layout, p: &BuildingPlan) -> String {
    let low = l.ground.iter().filter(|g| g.walkable()).map(|g| g.h0.min(g.h1)).fold(f32::INFINITY, f32::min);
    let mut s = String::new();
    for z in 0..l.h {
        for x in 0..l.w {
            let g = l.ground[(z * l.w + x) as usize];
            let c = if (x, z) == p.spawn {
                'S'
            } else if (x, z) == p.exit {
                'E'
            } else if p.cells.iter().any(|c| c.tile == (x, z)) {
                'C'
            } else {
                match g.kind {
                    GROUND_MASS => '#',
                    GROUND_VOID => ' ',
                    GROUND_STAIR => {
                        let down = if g.h1 < g.h0 { g.dir as usize } else { g.dir as usize ^ 1 };
                        ['>', '<', 'v', '^'][down]
                    }
                    _ => {
                        let k = (((g.h0 - low) as f64 / STOREY).round() as u32).min(35);
                        std::char::from_digit(k, 36).unwrap_or('?')
                    }
                }
            };
            s.push(c);
        }
        s.push('\n');
    }
    s
}
