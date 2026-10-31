//! Her marks (`docs/SECRETS.md` 1): what M. left to find her way by. She
//! spared her paper as she went down (she wrote on it) and marked the way
//! with what she had: a folded sheet, three bits stacked with a third the way she went,
//! her arrow in pencil on a wall, two strokes before a turn, a torn corner,
//! a chalk stroke low on a wall, reflective tape in the dark. One hand runs
//! through all of them (her arrow open, its left barb the longer); the
//! trails that are not hers are one stamp, a flat page every few tiles.
//!
//! Where each lies is planned here, on the story's stream (`lay_marks`):
//! each in sight of the last at its own distance (a small mark must be near
//! to be seen at all), the furthest step in reach where it fits; where no
//! small mark fits, she left a sheet.
use crate::level::archgen;
use crate::prelude::*;

/// How far in front of a wall a mark on it is seen from (its sight lines), m
pub const SEEN_OFF: f64 = 0.3;
/// How far off a wall's face a mark on it is drawn, m
pub const ON_WALL: f64 = 0.004;
/// The flat pages of a trail that is not hers lie this many tiles apart
/// (nearer only where the next could not be seen)
pub const FLAT_EVERY: usize = 2;

/// What one of her marks is. The trails that are not hers are all `Flat`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MarkKind {
    /// A sheet folded twice, her arrow on it in pencil
    Sheet,
    /// Three bits of what lies there, a small one on a large one and a third beside them the way she went
    Stack,
    /// Her arrow in pencil on a wall beside the way, at hip height
    Pencil,
    /// Two strokes on the wall ahead where her way turns, the upper set toward the turn
    Turn,
    /// A torn corner of her paper against a wall, its tip the way she went
    Corner,
    /// A chalk stroke low on a wall, leaning the way she went
    Chalk,
    /// A strip of reflective tape on a wall in the dark: a glint in your lamp
    Tape,
    /// A whole flat page with a heavy black arrow: not hers
    Flat,
}

/// Her kinds, in the order they first appear
pub const HER_KINDS: [MarkKind; 7] =
    [MarkKind::Sheet, MarkKind::Stack, MarkKind::Pencil, MarkKind::Turn, MarkKind::Corner, MarkKind::Chalk, MarkKind::Tape];

impl MarkKind {
    /// How near the last mark it must lie to be seen from there at all, m
    /// (a sheet or a flat page: as far as the spacing lets it)
    pub fn reach(self) -> f64 {
        match self {
            MarkKind::Sheet | MarkKind::Flat => f64::INFINITY,
            MarkKind::Stack => 12.0,
            MarkKind::Pencil | MarkKind::Turn => 9.0,
            MarkKind::Chalk => 7.0,
            MarkKind::Corner => 6.0,
            MarkKind::Tape => 16.0,
        }
    }
    /// The first level she marked her way so
    pub fn from_level(self) -> i32 {
        match self {
            MarkKind::Sheet | MarkKind::Stack | MarkKind::Flat => 1,
            MarkKind::Pencil => 2,
            MarkKind::Turn => 3,
            MarkKind::Corner | MarkKind::Chalk => 5,
            MarkKind::Tape => 7,
        }
    }
    /// It is on a wall (else on the floor)
    pub fn on_wall(self) -> bool {
        matches!(self, MarkKind::Pencil | MarkKind::Turn | MarkKind::Chalk | MarkKind::Tape)
    }
    /// The name of the headless view of it (`--views`)
    pub fn view_name(self) -> &'static str {
        match self {
            MarkKind::Sheet => "mark_sheet",
            MarkKind::Stack => "mark_stack",
            MarkKind::Pencil => "mark_pencil",
            MarkKind::Turn => "mark_turn",
            MarkKind::Corner => "mark_corner",
            MarkKind::Chalk => "mark_chalk",
            MarkKind::Tape => "mark_tape",
            MarkKind::Flat => "mark_flat",
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            MarkKind::Sheet => "sheet",
            MarkKind::Stack => "stack",
            MarkKind::Pencil => "pencil",
            MarkKind::Turn => "turn",
            MarkKind::Corner => "corner",
            MarkKind::Chalk => "chalk",
            MarkKind::Tape => "tape",
            MarkKind::Flat => "flat",
        }
    }
}

/// How many of her marks on a level are sheets (she spared her paper as
/// she went down)
pub fn sheet_share(level: i32) -> f64 {
    match level {
        ..=3 => 0.75,
        4..=6 => 0.5,
        7..=10 => 0.25,
        _ => 0.1,
    }
}

/// One mark as planned: pure data.
#[derive(Clone, Debug, PartialEq)]
pub struct PlanMark {
    /// Its step in the walk
    pub step: usize,
    pub kind: MarkKind,
    /// Where it is: on the floor, or on a wall's face (`normal` out of the
    /// wall, `y` up from the floor)
    pub x: f64,
    pub z: f64,
    pub y: f64,
    /// The way she went from it (a yaw: forward is (−sin, −cos))
    pub yaw: f64,
    pub normal: (f64, f64),
}

impl PlanMark {
    /// Where it is seen from in its sight lines: itself on the floor, a
    /// little in front of it on a wall
    pub fn seen_at(&self) -> (f64, f64) {
        (self.x + self.normal.0 * SEEN_OFF, self.z + self.normal.1 * SEEN_OFF)
    }
}

/// The yaw of the way from (x, z) toward (tx, tz) (forward is (−sin, −cos))
fn yaw_to(x: f64, z: f64, tx: f64, tz: f64) -> f64 {
    (-(tx - x)).atan2(-(tz - z))
}

/// The tile step from `a` to `b` (one of the four ways)
fn way(a: (i32, i32), b: (i32, i32)) -> (i32, i32) {
    ((b.0 - a.0).signum(), (b.1 - a.1).signum())
}

impl Game {
    /// The face of the wall from (x, z) the way `side` (a tile step), at
    /// height `y` over the floor, within `max` m: where it is, if it is a
    /// flat face (the same distance a hand either side along it) of a wall
    /// that stands where it was drawn (not a partition that leans), and on
    /// the Building's floors one of building, never a railing.
    pub fn mark_wall(&self, x: f64, z: f64, side: (i32, i32), y: f64, max: f64) -> Option<(f64, f64)> {
        let s = self.solids.as_ref()?;
        let dir = archgen::DIRS.iter().position(|d| d[0] == side.0 && d[1] == side.1)?;
        let fy = self.floor_y_at(x, z) + y;
        let ray = |ax: f64, az: f64| archgen::ray_axis(s, self.to_u(ax), self.to_v(az), dir, max / CELL, fy) * CELL;
        let d = ray(x, z);
        if !d.is_finite() || d < 0.25 {
            return None;
        }
        // flat: the same face a hand either side along it
        let (ax, az) = (-side.1 as f64, side.0 as f64);
        for k in [-0.3, 0.3] {
            let e = ray(x + ax * k, z + az * k);
            if !e.is_finite() || (e - d).abs() > 0.01 {
                return None;
            }
        }
        let (fx, fz) = (x + side.0 as f64 * d, z + side.1 as f64 * d);
        // not a partition that leans (its solids are steps inside the drawn wall)
        let (u, v) = (self.to_u(fx), self.to_v(fz));
        let leans =
            self.angled.iter().any(|w| w.parts.iter().any(|(_, b)| u > b[0] - 0.5 && u < b[2] + 0.5 && v > b[1] - 0.5 && v < b[3] + 0.5));
        if leans {
            return None;
        }
        // on the Building's floors: building, never the void's railing
        if !self.ground.is_empty() {
            let (tx, tz) = (self.tx(fx + side.0 as f64 * 0.05), self.tz(fz + side.1 as f64 * 0.05));
            let i = (tz * self.w + tx) as usize;
            if self.ground.get(i).is_none_or(|g| g.kind != archgen::GROUND_MASS) {
                return None;
            }
        }
        Some((fx, fz))
    }

    /// Whether the walk turns at `step` where three or more ways meet
    fn turns_at(&self, walk: &[(i32, i32)], step: usize) -> bool {
        if step == 0 || step + 1 >= walk.len() {
            return false;
        }
        let (a, b, c) = (walk[step - 1], walk[step], walk[step + 1]);
        let open = [(0, -1), (1, 0), (0, 1), (-1, 0)].iter().filter(|(dx, dz)| !self.is_wall_tile(b.0 + dx, b.1 + dz)).count();
        way(a, b) != way(b, c) && open >= 3
    }

    /// A mark of `kind` at `step` of `walk`, if one fits there: a sheet or a
    /// flat page anywhere on the floor; a stack on open floor; a corner
    /// against a wall; a pencil arrow, chalk or tape on a wall beside the
    /// way (her left hand first), tape only in the dark; a turn on the wall
    /// ahead where the way turns. Draws from `r` only while it fits.
    pub(crate) fn mark_at(&self, walk: &[(i32, i32)], step: usize, kind: MarkKind, r: &mut SplitMix64) -> Option<PlanMark> {
        let n = walk.len();
        let t = walk[step];
        let (cx, cz) = (self.wx(t.0), self.wz(t.1));
        let ahead = walk[(step + 2).min(n - 1)];
        let (ax, az) = (self.wx(ahead.0), self.wz(ahead.1));
        // the way through this tile
        let on = if step + 1 < n { way(t, walk[step + 1]) } else { way(walk[step.saturating_sub(1)], t) };
        let came = if step > 0 { way(walk[step - 1], t) } else { on };
        let floor = |x: f64, z: f64, yaw: f64| PlanMark { step, kind, x, z, y: 0.0, yaw, normal: (0.0, 0.0) };
        let walls_beside = [(0, -1), (1, 0), (0, 1), (-1, 0)].iter().filter(|(dx, dz)| self.is_wall_tile(t.0 + dx, t.1 + dz)).count();
        match kind {
            MarkKind::Sheet => {
                let (x, z) = (cx + r.range(-0.35, 0.35), cz + r.range(-0.35, 0.35));
                Some(floor(x, z, yaw_to(x, z, ax, az) + r.range(-0.12, 0.12)))
            }
            MarkKind::Flat => Some(floor(cx, cz, yaw_to(cx, cz, ax, az))),
            MarkKind::Stack => {
                // open floor: a hall, a deck, a field of pillars
                if walls_beside > 1 {
                    return None;
                }
                let (x, z) = (cx + r.range(-0.4, 0.4), cz + r.range(-0.4, 0.4));
                Some(floor(x, z, yaw_to(x, z, ax, az) + r.range(-0.2, 0.2)))
            }
            MarkKind::Corner | MarkKind::Pencil | MarkKind::Chalk | MarkKind::Tape => {
                if kind == MarkKind::Tape && self.region_light_at(t.0, t.1) > 0.15 {
                    return None;
                }
                // her left hand first: the wall on the left of the way, then the right
                let left = (on.1, -on.0);
                let right = (-on.1, on.0);
                let along = r.range(-0.6, 0.6);
                let (px, pz) = (cx + on.0 as f64 * along, cz + on.1 as f64 * along);
                let y = match kind {
                    MarkKind::Pencil => r.range(0.9, 1.08),
                    MarkKind::Chalk => r.range(0.1, 0.2),
                    MarkKind::Tape => r.range(1.05, 1.25),
                    _ => 0.3,
                };
                for side in [left, right] {
                    let Some((fx, fz)) = self.mark_wall(px, pz, side, y, CELL * 0.55) else { continue };
                    let normal = (-side.0 as f64, -side.1 as f64);
                    let yaw = yaw_to(cx, cz, ax, az);
                    if kind == MarkKind::Corner {
                        // on the floor against it
                        let (x, z) = (fx + normal.0 * 0.07, fz + normal.1 * 0.07);
                        return Some(floor(x, z, yaw + r.range(-0.15, 0.15)));
                    }
                    return Some(PlanMark { step, kind, x: fx + normal.0 * ON_WALL, z: fz + normal.1 * ON_WALL, y, yaw, normal });
                }
                None
            }
            MarkKind::Turn => {
                if !self.turns_at(walk, step) {
                    return None;
                }
                // on the wall ahead as you come to the turn
                let (fx, fz) = self.mark_wall(cx, cz, came, 1.0, CELL * 0.55)?;
                let normal = (-came.0 as f64, -came.1 as f64);
                let n2 = walk[step + 1];
                let yaw = yaw_to(cx, cz, self.wx(n2.0), self.wz(n2.1));
                let y = r.range(0.95, 1.08);
                Some(PlanMark { step, kind, x: fx + normal.0 * ON_WALL, z: fz + normal.1 * ON_WALL, y, yaw, normal })
            }
        }
    }

    /// Which kind she left next on this level: a sheet as often as
    /// `sheet_share` says, else one of the others she used by then (a turn
    /// only where the way turns: `lay_marks`). One draw.
    fn want_kind(&self, r: &mut SplitMix64) -> MarkKind {
        let u = r.next();
        let level = self.level_num;
        if u < sheet_share(level) {
            return MarkKind::Sheet;
        }
        let others: Vec<MarkKind> =
            HER_KINDS.iter().copied().filter(|k| !matches!(k, MarkKind::Sheet | MarkKind::Turn) && k.from_level() <= level).collect();
        if others.is_empty() {
            return MarkKind::Sheet;
        }
        let v = (u - sheet_share(level)) / (1.0 - sheet_share(level));
        others[((v * others.len() as f64) as usize).min(others.len() - 1)]
    }

    /// Marks along `walk` from step `first` to step `last` at most, each
    /// where the last can be seen from, at most `k` steps on, never on the
    /// `anchor` step (her camp, which is its own mark, seen from the middle
    /// of its tile), never on a flight's steps. Hers (`hers`): the first a
    /// sheet, then the kind she left next (`want_kind`) at the furthest step
    /// in reach where it fits and is seen at its own distance, a turn where
    /// the way turns, a sheet where no other fits; on the floor where a kind
    /// first appears, its first lies with a sheet beside it. Not hers: a
    /// flat page every `FLAT_EVERY` steps, mid-floor, where it can be seen.
    pub(crate) fn lay_marks(
        &self,
        walk: &[(i32, i32)],
        first: usize,
        last: usize,
        anchor: Option<usize>,
        k: usize,
        hers: bool,
        r: &mut SplitMix64,
    ) -> Vec<PlanMark> {
        let on_stair = |j: usize| {
            let (x, z) = walk[j];
            self.ground.get((z * self.w + x) as usize).is_some_and(|g| g.kind == archgen::GROUND_STAIR)
        };
        let base = if hers { MarkKind::Sheet } else { MarkKind::Flat };
        let mut marks: Vec<PlanMark> = vec![];
        let Some(m0) = self.mark_at(walk, first, base, r) else { return marks };
        let (mut px, mut pz) = m0.seen_at();
        marks.push(m0);
        let mut cur = first;
        let k = if hers { k } else { k.min(FLAT_EVERY) };
        while cur < last {
            let mut top = (cur + k).min(last);
            if let Some(a) = anchor.filter(|&a| cur < a && top > a) {
                top = a;
            }
            // her camp, seen from here: it is the next mark
            if anchor == Some(top) {
                let (x, z) = (self.wx(walk[top].0), self.wz(walk[top].1));
                if self.los(px, pz, x, z) {
                    (px, pz, cur) = (x, z, top);
                    continue;
                }
            }
            let want = if hers { self.want_kind(r) } else { base };
            // what she might leave here, in turn: the kind she meant to, then
            // the others she used by then (a sheet only when none fits)
            // (in the rooms a stack last: bits on a floor read as litter;
            // on the Building's decks it is what there is)
            let mut order = vec![want];
            if want != base {
                let pref: &[MarkKind] = if self.ground.is_empty() {
                    &[MarkKind::Pencil, MarkKind::Chalk, MarkKind::Corner, MarkKind::Tape, MarkKind::Stack]
                } else {
                    &[MarkKind::Stack, MarkKind::Pencil]
                };
                order.extend(pref.iter().copied().filter(|k| *k != want && k.from_level() <= self.level_num));
            }
            let mut placed: Option<PlanMark> = None;
            let mut j = top;
            while j > cur && placed.is_none() {
                if on_stair(j) || Some(j) == anchor {
                    j -= 1;
                    continue;
                }
                // a turn's mark where the way turns
                let turn = want != base && self.level_num >= MarkKind::Turn.from_level() && self.turns_at(walk, j);
                for &kind in std::iter::once(&MarkKind::Turn).filter(|_| turn).chain(order.iter()) {
                    let mut rr = *r;
                    if let Some(m) = self.mark_at(walk, j, kind, &mut rr) {
                        let (sx, sz) = m.seen_at();
                        if hypot2(sx - px, sz - pz) <= kind.reach() && self.los(px, pz, sx, sz) {
                            *r = rr;
                            placed = Some(m);
                            break;
                        }
                    }
                }
                j -= 1;
            }
            // where no other kind fits: a sheet, as she began
            if placed.is_none() && want != base {
                let mut j = top;
                while j > cur && placed.is_none() {
                    if !on_stair(j) && Some(j) != anchor {
                        let mut rr = *r;
                        if let Some(m) = self.mark_at(walk, j, base, &mut rr) {
                            let (sx, sz) = m.seen_at();
                            if self.los(px, pz, sx, sz) {
                                *r = rr;
                                placed = Some(m);
                            }
                        }
                    }
                    j -= 1;
                }
            }
            let m = match placed {
                Some(m) => m,
                None => {
                    // nothing seen from here: the next step, whatever it is
                    let j = cur + 1;
                    if Some(j) == anchor {
                        (px, pz, cur) = (self.wx(walk[j].0), self.wz(walk[j].1), j);
                        continue;
                    }
                    match self.mark_at(walk, j, base, r) {
                        Some(m) => m,
                        None => break,
                    }
                }
            };
            (px, pz) = m.seen_at();
            cur = m.step;
            marks.push(m);
        }
        // where a kind first appears, its first lies with a sheet beside it
        if hers {
            for kind in HER_KINDS.iter().copied().filter(|k| *k != MarkKind::Sheet && k.from_level() == self.level_num) {
                let Some(i) = marks.iter().position(|m| m.kind == kind) else { continue };
                let m = &marks[i];
                let (sx, sz) = m.seen_at();
                let (fx, fz) = (-m.yaw.sin(), -m.yaw.cos());
                // on the floor below it (a wall's mark), or a step short of it
                let (x, z) =
                    if kind.on_wall() { (sx + m.normal.0 * 0.15, sz + m.normal.1 * 0.15) } else { (m.x - fx * 0.45, m.z - fz * 0.45) };
                let sheet =
                    PlanMark { step: m.step, kind: MarkKind::Sheet, x, z, y: 0.0, yaw: m.yaw + r.range(-0.12, 0.12), normal: (0.0, 0.0) };
                marks.insert(i, sheet);
            }
        }
        marks
    }
}
