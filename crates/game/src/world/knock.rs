//! A knock at a sealed door (`docs/MEGASTRUCTURE.md`, G5). Along the
//! Well's galleries the residents' doors are sealed from inside, numbered,
//! some marked red by the cleaners, some taped round (`building_dress`);
//! `[E] KNOCK` raps on one. In the quiet, now and then
//! a resident answers without opening: a bottle of almond water or a battery
//! is pushed out through a hatch at the foot of the door, or a folded note
//! is slid under it (a word of the way from that door: the nearest cell, or
//! the way down). A door answers at most once, and most stay silent. A door
//! the cleaners marked answers wrong: it knocks back in your own rhythm,
//! then is still, and the self-assembly comes within seconds instead of
//! after its quiet. A door taped round never answers. While the siren
//! sounds or the fog is up nobody answers at all, and no hatch is opened.
//!
//! Which doors answer and what each gives is planned after the build, on a
//! stream of its own seeded from the floor's seed (`KNOCK_SALT`): no draw of
//! the build moves, and nothing is drawn from the per-frame stream as the
//! floor starts. A knock's rhythm and the answer's timing are drawn from the
//! per-frame stream when you knock, and its sounds play from the door. What
//! is taken is kept for the run as the hiding places' things are
//! (`secrets_taken`, `knock:<level>:<door>`): a retry or a Continue finds
//! that door has answered.
use crate::base::records::Pickup;
use crate::level::archgen::{DIRS, GROUND_STAIR};
use crate::prelude::*;

/// The salt of the stream the knock plan of a floor draws from
pub const KNOCK_SALT: u64 = 0x4B0C_D002_A115_0001;
/// How many of a floor's ordinary sealed doors answer (of some seventy:
/// the rest stay silent)
pub const ANSWERING: usize = 4;
/// What an answering door gives: almond water below `GIFT_WATER`, a battery
/// below `GIFT_BATTERY`, else a folded note (half of them the nearest cell,
/// half the way down)
pub const GIFT_WATER: f64 = 0.4;
pub const GIFT_BATTERY: f64 = 0.7;
/// Seconds from your knock to someone at the door, and from then to the
/// thing coming out
pub const ANSWER_AFTER: (f64, f64) = (2.0, 3.4);
pub const OUT_AFTER: f64 = 0.9;
/// Seconds from your knock to the wrong door's knock back, and from its
/// knock back to the siren
pub const BACK_AFTER: (f64, f64) = (1.3, 2.1);
pub const WRONG_WAKE: (f64, f64) = (3.0, 6.0);
/// The gaps between your three raps, seconds
pub const RAP: (f64, f64) = (0.2, 0.55);
/// How far off what happens at a door is still heard, metres
const EARSHOT: f64 = 40.0;
/// A thing pushed out lies still this long before it can be picked up, s
const SETTLE: f64 = 1.6;

/// How a sealed door is marked
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mark {
    /// numbered, nothing else
    #[default]
    Plain,
    /// the cleaners' red cross painted over it
    Red,
    /// taped round its edges
    Taped,
}

/// What an answering door gives
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gift {
    Water,
    Battery,
    /// a folded note: the nearest cell from this door
    Cell,
    /// a folded note: the way down from this door
    WayDown,
}

impl Gift {
    pub fn is_note(self) -> bool {
        matches!(self, Gift::Cell | Gift::WayDown)
    }
}

/// A sealed door along a gallery.
#[derive(Clone, Debug)]
pub struct KnockDoor {
    /// the walk's tile in front of it, and the side of that tile it is on (a
    /// `DIRS` index, toward the building)
    pub tile: (i32, i32),
    pub side: usize,
    /// its plate
    pub number: u32,
    pub mark: Mark,
    /// its foot: the middle of the doorway's sill on the face, world
    pub foot: Vec3,
    /// out of the face (toward the walk), and to the right seen from outside
    pub normal: (f64, f64),
    pub right: (f64, f64),
    /// what it gives when it answers (the plan), and for a note its words
    pub gift: Option<Gift>,
    pub words: String,
    /// where the thing comes to rest: metres out from the face, aside, its turn
    pub lie: (f64, f64, f64),
    /// it has answered (or is answering)
    pub answered: bool,
}

impl KnockDoor {
    /// A point `out` metres out from its face, `aside` to the right, `y` up
    /// from its sill
    pub fn point(&self, out: f64, aside: f64, y: f64) -> Vec3 {
        Vec3::new(
            self.foot.x + self.normal.0 * out + self.right.0 * aside,
            self.foot.y + y,
            self.foot.z + self.normal.1 * out + self.right.1 * aside,
        )
    }
    /// Where you stand to knock (an eye), and where its sounds come from
    pub fn stand(&self) -> Vec3 {
        self.point(0.9, 0.0, EYE)
    }
    pub fn voice(&self) -> Vec3 {
        self.point(0.0, 0.0, 1.2)
    }
    /// The turn that faces its own +z out of the face
    pub fn yaw(&self) -> f64 {
        self.normal.0.atan2(self.normal.1)
    }
    /// Its key in the run's record of what was taken
    pub fn key(level: i32, i: usize) -> String {
        format!("knock:{level}:{i}")
    }
}

/// What is to come at a door after a knock
#[derive(Clone, Debug)]
pub enum Due {
    /// someone at the door; the thing pushed out after
    Answer(usize),
    /// the wrong door knocks back in these gaps
    KnockBack(usize, Vec<f64>),
}

/// A thing pushed out at a door, as it comes out and lies there.
#[derive(Clone, Debug)]
pub struct Given {
    pub door: usize,
    pub gift: Gift,
    pub obj: Obj,
    /// the hatch it came through (its frame, and the flap's hinge)
    pub hatch: Option<(Obj, Obj)>,
    /// seconds since it began to come out (below 0: someone at the door)
    pub t: f64,
    /// the pickup it became, once still: (water, index in its list)
    pub pickup: Option<(bool, usize)>,
    pub taken: bool,
}

/// The knocking on this floor
#[derive(Clone, Debug, Default)]
pub struct Knocks {
    pub doors: Vec<KnockDoor>,
    /// the cells' plates, in the plan's order (`building_dress`)
    pub cell_plates: Vec<String>,
    pub due: Vec<(f64, Due)>,
    pub given: Vec<Given>,
    /// the gaps of your last knock
    pub rhythm: Vec<f64>,
    /// the plan is made for this floor
    pub planned: bool,
}

/// A small number in words, as a hand writes it
fn count_word(n: usize) -> String {
    const W: [&str; 11] = ["no", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten"];
    W.get(n).map(|s| s.to_string()).unwrap_or_else(|| n.to_string())
}

/// The way from a door to somewhere, as a note gives it
struct Way {
    /// the first step from the door, facing it: "left", "right" or "out"
    first: &'static str,
    /// out first, then which way along
    then: Option<&'static str>,
    /// the runs of stairs down and up on the way
    down: usize,
    up: usize,
    /// the storey painted there
    storey: i32,
}

impl Way {
    /// Which way from the door, in hand `h` (0 in words, 1 terse, 2 told)
    fn dir(&self, h: usize) -> String {
        let (lead, from) = match h {
            0 => ("", "this door"),
            1 => ("", "here"),
            _ => ("go ", "the door"),
        };
        match (self.first, self.then) {
            ("out", Some(s)) => format!("{lead}out from {from}, then {s}"),
            ("out", None) => format!("{lead}straight out from {from}"),
            (s, _) => format!("{lead}{s} from {from}"),
        }
    }
    /// How many flights, in hand `h` (1: in figures, the rest in words)
    fn flights(&self, h: usize) -> String {
        let fl = |n: usize| if n == 1 { "flight" } else { "flights" };
        match (h, self.down, self.up) {
            (1, 0, 0) => "no stairs".to_string(),
            (1, d, 0) => format!("{d} {} down", fl(d)),
            (1, 0, u) => format!("{u} {} up", fl(u)),
            (1, d, u) => format!("{d} {} down, {u} up", fl(d)),
            (_, 0, 0) => "along the gallery".to_string(),
            (_, d, 0) => format!("down {} {}", count_word(d), fl(d)),
            (_, 0, u) => format!("up {} {}", count_word(u), fl(u)),
            (_, d, u) => format!("down {} {} and up {}", count_word(d), fl(d), count_word(u)),
        }
    }
}

/// `s` with its first letter a capital (a sentence's start)
fn cap(s: &str) -> String {
    let mut c = s.chars();
    c.next().map(|f| f.to_uppercase().chain(c).collect()).unwrap_or_default()
}

/// Ease out over 0..1
fn ease(t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t) * (1.0 - t)
}

impl Game {
    /// The knock plan of this floor, then what the run has taken from its
    /// doors: after the build (and again after a Continue's save is read).
    /// Pure: the plan from the floor's own stream and the floor as built.
    pub fn fit_knocks(&mut self) {
        if !self.knocks.planned {
            self.plan_knocks();
        }
        let level = self.level_num;
        for i in 0..self.knocks.doors.len() {
            self.knocks.doors[i].answered = self.secrets_taken.contains_key(&KnockDoor::key(level, i));
        }
    }

    /// Which doors answer and what each gives: the `ANSWERING` ordinary
    /// doors with the lowest lots; four draws a door whatever it is, so a
    /// door's lot never moves another's.
    fn plan_knocks(&mut self) {
        self.knocks.planned = true;
        if self.knocks.doors.is_empty() {
            return;
        }
        let mut r = SplitMix64::new(self.current_seed, KNOCK_SALT);
        let lots: Vec<[f64; 4]> = (0..self.knocks.doors.len()).map(|_| [r.next(), r.next(), r.next(), r.next()]).collect();
        let mut plain: Vec<usize> = (0..self.knocks.doors.len()).filter(|&i| self.knocks.doors[i].mark == Mark::Plain).collect();
        plain.sort_by(|&a, &b| lots[a][0].total_cmp(&lots[b][0]).then(a.cmp(&b)));
        plain.truncate(ANSWERING);
        for i in plain {
            let [_, g, w, l] = lots[i];
            let mut gift = match g {
                g if g < GIFT_WATER => Gift::Water,
                g if g < GIFT_BATTERY => Gift::Battery,
                g if (g - GIFT_BATTERY) / (1.0 - GIFT_BATTERY) < 0.5 => Gift::Cell,
                _ => Gift::WayDown,
            };
            let mut words = String::new();
            if gift.is_note() {
                match self.knock_words(i, gift, w) {
                    Some(s) => words = s,
                    // (no way from this door: a bottle instead)
                    None => gift = Gift::Water,
                }
            }
            let out = match gift {
                Gift::Water => 0.55,
                Gift::Battery => 0.5,
                _ => 0.42,
            };
            let d = &mut self.knocks.doors[i];
            d.gift = Some(gift);
            d.words = words;
            d.lie = (out, (l - 0.5) * 0.3, (l * 7.0).fract() * 0.8 - 0.4);
        }
    }

    /// The way from door `i`'s walk to `goal`: which way from the door,
    /// how many flights down and up, and the storey there
    fn knock_way(&self, i: usize, goal: (i32, i32)) -> Option<Way> {
        let door = &self.knocks.doors[i];
        let d = self.story_walk_dist(goal);
        let at = |p: (i32, i32)| if self.in_grid(p.0, p.1) { d[self.t_idx(p.0, p.1)] } else { -1 };
        if at(door.tile) < 0 {
            return None;
        }
        // down the distances to the goal, neighbours in the fixed order `DIRS`
        let mut path = vec![door.tile];
        let mut cur = door.tile;
        while at(cur) > 0 {
            let next = DIRS.iter().map(|d| (cur.0 + d[0], cur.1 + d[1])).find(|&p| at(p) == at(cur) - 1)?;
            path.push(next);
            cur = next;
        }
        // which way from the door: facing it, its right is (−fz, fx)
        let f = (DIRS[door.side][0], DIRS[door.side][1]);
        let side_of = |a: (i32, i32), b: (i32, i32)| {
            let e = (b.0 - a.0, b.1 - a.1);
            if e == (-f.1, f.0) {
                "right"
            } else if e == (f.1, -f.0) {
                "left"
            } else {
                "out"
            }
        };
        // (the goal at this very door: nothing worth writing)
        let first = side_of(path[0], *path.get(1)?);
        let then = path.windows(2).map(|w| side_of(w[0], w[1])).find(|s| *s != "out");
        // the flights on the way: each run of stairs, down or up
        let (mut down, mut up) = (0usize, 0usize);
        let stair = |p: (i32, i32)| self.ground_or_mass(p.0, p.1).kind == GROUND_STAIR;
        let mut k = 0;
        while k < path.len() {
            if !stair(path[k]) {
                k += 1;
                continue;
            }
            let from = self.floor_y_of(path[k.saturating_sub(1)].0, path[k.saturating_sub(1)].1);
            while k < path.len() && stair(path[k]) {
                k += 1;
            }
            let to = path.get(k).map(|p| self.floor_y_of(p.0, p.1)).unwrap_or(from);
            if to < from - 0.5 {
                down += 1;
            } else if to > from + 0.5 {
                up += 1;
            }
        }
        let style = self.building.as_ref()?.style;
        let storey = crate::world::building_dress::storey_base(style)
            + (self.floor_y_of(goal.0, goal.1) / crate::level::building::STOREY).round() as i32;
        Some(Way { first, then: if first == "out" { then } else { None }, down, up, storey })
    }

    /// A folded note's words from door `i`: the nearest cell by the walk, or
    /// the way down, as residents write them (`w` picks the hand)
    fn knock_words(&self, i: usize, gift: Gift, w: f64) -> Option<String> {
        let plan = self.building.as_ref()?;
        let hand = ((w * 3.0) as usize).min(2);
        match gift {
            Gift::Cell => {
                let from = self.story_walk_dist(self.knocks.doors[i].tile);
                let (ci, _) = plan
                    .cells
                    .iter()
                    .enumerate()
                    .filter_map(|(ci, c)| from.get(self.t_idx(c.tile.0, c.tile.1)).filter(|&&s| s >= 0).map(|&s| (ci, s)))
                    .min_by_key(|&(ci, s)| (s, ci))?;
                let plate = self.knocks.cell_plates.get(ci).cloned().unwrap_or_default();
                let w = self.knock_way(i, plan.cells[ci].tile)?;
                let s = w.storey;
                Some(match hand {
                    0 => format!("{plate} is open. {}, {}, storey {s}. The lamp over it is lit.", cap(&w.dir(0)), w.flights(0)),
                    1 => format!("Nearest cell {plate}. {}. {}. {s}. Shut it behind you.", cap(&w.dir(1)), cap(&w.flights(1))),
                    _ => format!("{}, {}, to {s}: cell {plate}. Go before the siren.", cap(&w.dir(2)), w.flights(0)),
                })
            }
            _ => {
                let w = self.knock_way(i, plan.exit)?;
                let s = w.storey;
                Some(match hand {
                    0 => format!("Down is under the tower's lit windows. {}, {}, to storey {s}.", cap(&w.dir(0)), w.flights(0)),
                    1 => format!("Way down at {s}, the foot of the lit tower. {}. {}.", cap(&w.dir(1)), cap(&w.flights(1))),
                    _ => format!("Down: {}, {}, to {s}. Go while it is quiet. Do not knock again.", w.dir(2), w.flights(0)),
                })
            }
        }
    }

    /// Your three raps' rhythm (the two gaps between them, `RAP`), drawn
    /// from the per-frame stream as you knock, and kept as your rhythm
    /// (`Knocks::rhythm`): at a resident's door and at your cell's alike
    pub fn draw_raps(&mut self) -> Vec<f64> {
        let gaps = vec![RAP.0 + native_rnd() * (RAP.1 - RAP.0), RAP.0 + native_rnd() * (RAP.1 - RAP.0)];
        self.knocks.rhythm = gaps.clone();
        gaps
    }

    /// Knock at door `i` (`[E] KNOCK`): three raps in a rhythm of your own,
    /// and what answers, if anything (the module's rule)
    pub fn knock_at(&mut self, i: usize) {
        let Some(door) = self.knocks.doors.get(i).cloned() else { return };
        let gaps = self.draw_raps();
        let v = door.voice();
        if self.ac.is_some() {
            let g2 = gaps.clone();
            self.play_from(v.x, v.y, v.z, 3.0, 1.0, 0.0, move |g| g.sfx_knock(&g2, false));
        }
        // once a door; nobody answers while the siren sounds or the fog is up
        if door.answered || self.assembly.up() {
            return;
        }
        match (door.mark, door.gift) {
            (Mark::Red, _) => {
                self.knocks.doors[i].answered = true;
                let t = BACK_AFTER.0 + native_rnd() * (BACK_AFTER.1 - BACK_AFTER.0);
                self.knocks.due.push((t, Due::KnockBack(i, gaps)));
            }
            (Mark::Plain, Some(_)) => {
                self.knocks.doors[i].answered = true;
                let t = ANSWER_AFTER.0 + native_rnd() * (ANSWER_AFTER.1 - ANSWER_AFTER.0);
                self.knocks.due.push((t, Due::Answer(i)));
            }
            // taped round: never; an ordinary door: silence
            _ => {}
        }
    }

    /// Whether what happens at `p` is heard from where you are
    fn in_earshot(&self, p: Vec3) -> bool {
        self.ac.is_some() && hypot2(p.x - self.player.pos.x, p.z - self.player.pos.z) < EARSHOT
    }

    /// One step of the knocking (the director's, on the Building's floors):
    /// answers on their way, things coming out, lying, taken
    pub fn update_knocks(&mut self, dt: f64) {
        if self.knocks.due.is_empty() && self.knocks.given.is_empty() {
            return;
        }
        for d in self.knocks.due.iter_mut() {
            d.0 -= dt;
        }
        let due: Vec<Due> = self.knocks.due.iter().filter(|d| d.0 <= 0.0).map(|d| d.1.clone()).collect();
        self.knocks.due.retain(|d| d.0 > 0.0);
        for d in due {
            // (the siren came first: nobody opens while it is up, and the
            // door may answer another time)
            let i = match &d {
                Due::Answer(i) | Due::KnockBack(i, _) => *i,
            };
            if self.assembly.up() {
                self.knocks.doors[i].answered = false;
                continue;
            }
            match d {
                Due::KnockBack(i, gaps) => {
                    let v = self.knocks.doors[i].voice();
                    if self.in_earshot(v) {
                        self.play_from(v.x, v.y, v.z, 3.0, 1.0, 0.0, move |g| g.sfx_knock(&gaps, true));
                    }
                    self.director_note("knock:wrong");
                    self.assembly_called_early(WRONG_WAKE);
                }
                Due::Answer(i) => self.knock_answer(i, -OUT_AFTER),
            }
        }
        self.update_given(dt);
    }

    /// Door `i` answers: someone at it (at `t` below 0), then its thing
    /// pushed out (`t` from 0)
    fn knock_answer(&mut self, i: usize, t: f64) {
        let door = self.knocks.doors[i].clone();
        let Some(gift) = door.gift else { return };
        let obj = self.knock_thing(gift);
        obj.set_visible(t >= 0.0);
        self.world.add(&obj);
        let hatch = (!gift.is_note()).then(|| {
            let (frame, hinge) = self.knock_hatch();
            frame.position().copy(&door.point(0.006, 0.0, 0.0));
            frame.rotation().y = door.yaw();
            frame.set_visible((0.0..1.2).contains(&t));
            self.world.add(&frame);
            (frame, hinge)
        });
        self.knocks.given.push(Given { door: i, gift, obj, hatch, t, pickup: None, taken: false });
        self.director_note("knock:answer");
        let v = door.voice();
        if t < 0.0 && self.in_earshot(v) {
            self.play_from(v.x, v.y - 0.6, v.z, 2.5, 1.0, 0.3, |g| g.sfx_behind_door());
        }
        self.place_given(self.knocks.given.len() - 1);
    }

    /// The thing a door gives, as it lies: a bottle or a battery on its
    /// side, a note folded once (its frame: +z out of the door, on the floor)
    fn knock_thing(&self, gift: Gift) -> Obj {
        let outer = group();
        let inner = group();
        match gift {
            Gift::Water => {
                let body =
                    mesh(pg("Props__almond_bottle", cyl_geo!(0.09, 0.09, 0.3, 10)), lambert!({ color: 0x9fc4d8, emissive: 0x16323e }));
                inner.add(&body);
                let cap = mesh(pg("Props__almond_cap", cyl_geo!(0.05, 0.05, 0.06, 8)), &self.mats.metal);
                cap.position().y = 0.18;
                inner.add(&cap);
                inner.rotation().x = PI / 2.0;
                inner.position().y = 0.09;
            }
            Gift::Battery => {
                let body =
                    mesh(pg("Props__battery_body", cyl_geo!(0.06, 0.06, 0.18, 10)), lambert!({ color: 0xc79b3d, emissive: 0x2a1f08 }));
                inner.add(&body);
                let tip = mesh(pg("Props__battery_tip", cyl_geo!(0.025, 0.025, 0.03, 8)), lambert!({ color: 0xdad3c2 }));
                tip.position().y = 0.105;
                inner.add(&tip);
                inner.rotation().x = PI / 2.0;
                inner.position().y = 0.06;
            }
            _ => {
                // a sheet folded once: its under half flat on the floor, its
                // upper half sprung a little open on the fold
                let paper = lambert!({ color: 0xd6d0bf, side: DoubleSide });
                for (k, lift) in [(0.0, 0.0), (1.0, 0.3)] {
                    let leaf = mesh(plane_geo!(0.1, 0.145), &paper);
                    leaf.rotation().x = -PI / 2.0;
                    leaf.position().x = 0.05;
                    let hinge = group();
                    hinge.position().set(-0.05, 0.002 + k * 0.002, 0.0);
                    hinge.rotation().z = lift;
                    hinge.add(&leaf);
                    inner.add(&hinge);
                }
            }
        }
        outer.add(&inner);
        outer.user().gameplay = true;
        outer
    }

    /// A hatch at the foot of a sealed door: its dark opening and the flap
    /// on a hinge along its top (the frame: +z out of the door, on the sill)
    fn knock_hatch(&self) -> (Obj, Obj) {
        let frame = group();
        let hole = mesh(box_geo!(0.34, 0.22, 0.004), basic_mat!({ color: 0x0b0b0a }));
        hole.position().y = 0.15;
        frame.add(&hole);
        let hinge = group();
        hinge.position().set(0.0, 0.26, 0.004);
        let flap = mesh(box_geo!(0.34, 0.22, 0.012), lambert!({ color: 0x47504a }));
        flap.position().set(0.0, -0.11, 0.006);
        hinge.add(&flap);
        frame.add(&hinge);
        frame.user().gameplay = true;
        (frame, hinge)
    }

    /// Set given thing `k` where it is at its `t`: through the hatch (the
    /// flap up, the thing slid out, the flap down) or under the door
    fn place_given(&self, k: usize) {
        let gv = &self.knocks.given[k];
        let door = &self.knocks.doors[gv.door];
        let (out, aside, turn) = door.lie;
        let t = gv.t;
        let s = if gv.gift.is_note() { ease(t / 0.5) } else { ease((t - 0.1) / 0.5) };
        let start = if gv.gift.is_note() { -0.12 } else { -0.1 };
        let p = door.point(start + (out - start) * s, aside * s, 0.0);
        gv.obj.position().copy(&p);
        gv.obj.rotation().y = door.yaw() + turn * s;
        if let Some((frame, hinge)) = &gv.hatch {
            let a = if t < 0.15 {
                t / 0.15
            } else if t < 0.75 {
                1.0
            } else {
                1.0 - (t - 0.75) / 0.2
            };
            hinge.rotation().x = -1.25 * a.clamp(0.0, 1.0);
            frame.set_visible((0.0..1.0).contains(&t));
        }
    }

    /// The things coming out and lying at the doors: their sounds as they
    /// come, a pickup once still, what was taken kept for the run
    fn update_given(&mut self, dt: f64) {
        let level = self.level_num;
        // someone at a door when the siren starts goes away: no hatch is
        // opened while it is up (the door may answer another time)
        if self.assembly.up() {
            for gv in self.knocks.given.iter().filter(|gv| gv.t < 0.0) {
                gv.obj.set_visible(false);
                if let Some((frame, _)) = &gv.hatch {
                    frame.set_visible(false);
                }
            }
            for k in 0..self.knocks.given.len() {
                if self.knocks.given[k].t < 0.0 {
                    let d = self.knocks.given[k].door;
                    self.knocks.doors[d].answered = false;
                }
            }
            self.knocks.given.retain(|gv| gv.t >= 0.0);
        }
        for k in 0..self.knocks.given.len() {
            let (t0, gift, door) = {
                let gv = &mut self.knocks.given[k];
                let t0 = gv.t;
                gv.t += dt;
                (t0, gv.gift, gv.door)
            };
            let t = self.knocks.given[k].t;
            let foot = self.knocks.doors[door].point(0.1, 0.0, 0.15);
            let heard = self.in_earshot(foot);
            // the hatch opened, or the paper under the door; the thing out;
            // the hatch shut
            if t0 < 0.0 && t >= 0.0 {
                self.knocks.given[k].obj.set_visible(true);
                if heard {
                    self.play_from(foot.x, foot.y, foot.z, 2.0, 1.0, 0.0, move |g| g.sfx_hatch(gift, true));
                }
            }
            if t0 < 0.55 && t >= 0.55 && heard {
                self.play_from(foot.x, foot.y, foot.z, 2.0, 1.0, 0.0, move |g| g.sfx_pushed_out(gift));
            }
            if !gift.is_note() && t0 < 0.9 && t >= 0.9 && heard {
                self.play_from(foot.x, foot.y, foot.z, 2.0, 1.0, 0.0, move |g| g.sfx_hatch(gift, false));
            }
            if t0 < 1.3 {
                self.place_given(k);
            }
            if self.knocks.given[k].taken || t < SETTLE {
                continue;
            }
            let key = KnockDoor::key(level, door);
            match (gift, self.knocks.given[k].pickup) {
                // a bottle or a battery: one of the floor's pickups now
                // (`player.rs` gives it), its taking kept
                (Gift::Water | Gift::Battery, None) => {
                    let gv = &self.knocks.given[k];
                    let p = Pickup { g: gv.obj.clone(), taken: false, base: gv.obj.pos().y, phase: 0.0 };
                    let water = gift == Gift::Water;
                    let list = if water { &mut self.almonds } else { &mut self.battery_items };
                    list.push(p);
                    let idx = list.len() - 1;
                    self.knocks.given[k].pickup = Some((water, idx));
                }
                (Gift::Water | Gift::Battery, Some((water, idx))) => {
                    let list = if water { &self.almonds } else { &self.battery_items };
                    if list.get(idx).is_some_and(|p| p.taken) {
                        self.knocks.given[k].taken = true;
                        self.secrets_taken.insert(key, 1);
                        self.save_game();
                    }
                }
                // a note: read as it is picked up
                _ => {
                    let p = self.knocks.given[k].obj.pos();
                    let me = self.player.pos;
                    let (dx, dz) = (p.x - me.x, p.z - me.z);
                    let foot_y = me.y - EYE;
                    if dx * dx + dz * dz < 0.95 && (foot_y - p.y).abs() < 1.5 {
                        self.knocks.given[k].taken = true;
                        self.knocks.given[k].obj.set_visible(false);
                        self.sfx_pickup();
                        let words = self.knocks.doors[door].words.clone();
                        self.show_msg(&format!("A FOLDED NOTE — {words}"), 7.0);
                        self.secrets_taken.insert(key, 1);
                        self.save_game();
                    }
                }
            }
        }
    }

    /// For the evidence (`--knock-demo`): door `i`'s thing held where it is
    /// `t` seconds after it began to come out (0.35: the hatch open and it
    /// half out; 1.25 and on: lying still, the hatch shut)
    pub fn knock_lay_out(&mut self, i: usize, t: f64) -> bool {
        if self.knocks.doors.get(i).and_then(|d| d.gift).is_none() {
            return false;
        }
        self.knocks.doors[i].answered = true;
        match self.knocks.given.iter().position(|gv| gv.door == i) {
            Some(k) => {
                self.knocks.given[k].t = t;
                self.place_given(k);
            }
            None => self.knock_answer(i, t),
        }
        true
    }

    /// For the evidence: the first answering door of each kind of gift (a
    /// bottle, a battery, a note), each with a camera standing back from it
    /// and looking down at its foot: (name, door, [x, y, z, yaw, pitch])
    pub fn knock_cameras(&self) -> Vec<(String, usize, [f64; 5])> {
        let mut out = vec![];
        for (name, want) in [("water", Gift::Water), ("battery", Gift::Battery), ("note", Gift::Cell)] {
            let Some(i) = self.knocks.doors.iter().position(|d| d.gift.is_some_and(|g| g == want || (want.is_note() && g.is_note())))
            else {
                continue;
            };
            let d = &self.knocks.doors[i];
            let eye = d.point(1.75, -0.35, EYE);
            // facing the door: the look's forward is (−sin yaw, −cos yaw)
            let yaw = d.normal.0.atan2(d.normal.1);
            out.push((name.to_string(), i, [eye.x, eye.y, eye.z, yaw, -0.62]));
        }
        out
    }
}
