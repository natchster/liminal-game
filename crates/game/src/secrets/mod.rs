//! Secrets (`docs/design/SECRETS.md` 4–5): on each floor a few places where
//! something is put away off the way, each with a tell you can see before
//! you look, the same all run (`places`), and what is in them: mostly
//! ordinary things or nothing, now and then a thing that stays with you
//! (`things`, `keeps`).
//!
//! Planned inside the build once her trail is down (`place_secrets`, after
//! `place_story`: the props round them make room), on a stream of their own
//! (`SECRET_SALT`), never `rnd()`; which things lie on which floors is the
//! run's, from its seed alone (`run_things`). What was taken (a retry, a
//! Continue) is fitted afterwards (`fit_secrets`), as her page is.
//!
//! Attribution (CC BY-SA 3.0 / CC BY-SA; the full list in `docs/design/LORE.md`,
//! *Attribution and licensing*): SCP-330 by Dr Kondraki and "Please Take
//! Just One" by Umbrello (the dish), the first SCP-1162 (author unknown;
//! the hole), SCP-268 by Pair Of Ducks (the cap), SCP-1916 by Smapti,
//! SCP-714 by Arcalane, SCP-832 by Inglonias; the Backrooms wikis' Moth Jelly
//! (KingofGarbage), Liquid Silence (Atom_Rutherford), Cashew Water
//! (yoyoflux, Positive123; Taangeerine), Liquid Pain (ReyDay), Royal Rations
//! (Sariastuff, zakkun_); the Samosbor wiki's food concentrate
//! (ProfessorSacred); "The Midnight Game" (author unknown). Public domain:
//! Carroll's *Alice*, Persephone, Izanami, Rossetti's *Goblin Market*, folk
//! custom. The things, their places and what each does are this game's own.
pub mod dress;
pub mod keeps;
pub mod places;
pub mod things;

use crate::prelude::*;

/// Salt of the stream the secrets of a floor draw from
pub const SECRET_SALT: u64 = 0x5EC2_E7ED_0AB5_7A1D;
/// Salt of the stream that lays the run's things over its floors
pub const RUN_SALT: u64 = 0x7A4E_0F1E_5EED_0001;
/// The floors of the first descent the run's things are laid over
pub const RUN_FLOORS: i32 = 13;

/// Where something is put away (`places`)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PlaceKind {
    /// Over a ceiling tile that sits askew, a chair under it
    Tile,
    /// Behind a low vent grille hanging by two screws
    Vent,
    /// Taped under a filing cabinet's drawer that will not quite shut
    Drawer,
    /// Behind a radiator, a rag stuffed behind its fins (the Building's floors)
    Radiator,
    /// On the bench of a cell off the way (the Building's floors)
    Cell,
    /// The hole in the wall: reach in, and it gives back and takes
    Hole,
    /// Lying where nobody would leave it: the end of a dead end, a corner
    Corner,
    /// On a small table against a wall in an empty room
    Table,
    /// On a coat hook
    Hook,
}

impl PlaceKind {
    /// It is opened before what is in it shows (its own verb)
    pub fn closed(self) -> bool {
        matches!(self, PlaceKind::Tile | PlaceKind::Vent | PlaceKind::Drawer | PlaceKind::Radiator | PlaceKind::Hole)
    }
    pub fn name(self) -> &'static str {
        match self {
            PlaceKind::Tile => "tile",
            PlaceKind::Vent => "vent",
            PlaceKind::Drawer => "drawer",
            PlaceKind::Radiator => "radiator",
            PlaceKind::Cell => "cell",
            PlaceKind::Hole => "hole",
            PlaceKind::Corner => "corner",
            PlaceKind::Table => "table",
            PlaceKind::Hook => "hook",
        }
    }
}

/// What is put away
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Thing {
    PaleMushrooms,
    BrownMushrooms,
    Sweets,
    Seeds,
    Bag,
    Salt,
    Jelly,
    Ration,
    Medicine,
    Cap,
    Coin,
    YellowWater,
    Envelope,
    // what is ordinary
    Almond,
    Battery,
    Penny,
    Paper,
    Nothing,
}

/// The things that stay with you, in the order the run lays them out
pub const SPECIAL: [Thing; 13] = [
    Thing::PaleMushrooms,
    Thing::BrownMushrooms,
    Thing::Sweets,
    Thing::Seeds,
    Thing::Bag,
    Thing::Salt,
    Thing::Jelly,
    Thing::Ration,
    Thing::Medicine,
    Thing::Cap,
    Thing::Coin,
    Thing::YellowWater,
    Thing::Envelope,
];

impl Thing {
    /// It stays with you (else it is what floors hold anyway, or nothing)
    pub fn special(self) -> bool {
        SPECIAL.contains(&self)
    }
    /// Its name as the save and the notebook know it
    pub fn id(self) -> &'static str {
        match self {
            Thing::PaleMushrooms => "pale_mushrooms",
            Thing::BrownMushrooms => "brown_mushrooms",
            Thing::Sweets => "sweet",
            Thing::Seeds => "seeds",
            Thing::Bag => "bag",
            Thing::Salt => "salt",
            Thing::Jelly => "jelly",
            Thing::Ration => "ration",
            Thing::Medicine => "medicine",
            Thing::Cap => "cap",
            Thing::Coin => "coin",
            Thing::YellowWater => "yellow_water",
            Thing::Envelope => "envelope",
            Thing::Almond => "almond",
            Thing::Battery => "battery",
            Thing::Penny => "penny",
            Thing::Paper => "paper",
            Thing::Nothing => "nothing",
        }
    }
    pub fn from_id(id: &str) -> Option<Thing> {
        SPECIAL.iter().copied().find(|t| t.id() == id)
    }
    /// Where it may be put away, in order of preference
    pub fn places(self) -> &'static [PlaceKind] {
        use PlaceKind::*;
        match self {
            Thing::PaleMushrooms | Thing::BrownMushrooms | Thing::Seeds | Thing::Envelope => &[Corner],
            Thing::Sweets => &[Table],
            Thing::Bag => &[Radiator, Cell, Vent, Drawer],
            Thing::Salt => &[Cell, Drawer],
            Thing::Jelly => &[Tile],
            Thing::Ration => &[Cell, Radiator],
            Thing::Medicine => &[Vent, Drawer, Tile],
            Thing::Cap => &[Hook],
            Thing::Coin => &[Radiator, Vent, Drawer],
            Thing::YellowWater => &[Drawer, Table, Corner],
            _ => &[Tile, Vent, Drawer, Radiator, Cell],
        }
    }
    /// The floors of the first descent it may lie on (first, last), and on
    /// how many of them it lies in a run
    pub fn floors(self) -> (i32, i32, usize) {
        match self {
            Thing::PaleMushrooms => (4, 13, 2),
            Thing::BrownMushrooms => (6, 13, 2),
            Thing::Sweets => (5, 13, 2),
            Thing::Seeds => (5, 13, 1),
            Thing::Bag => (1, 13, 1),
            Thing::Salt => (1, 6, 1),
            Thing::Jelly => (5, 13, 1),
            Thing::Ration => (1, 3, 1),
            Thing::Medicine => (5, 13, 1),
            Thing::Cap => (5, 13, 1),
            Thing::Coin => (1, 13, 1),
            Thing::YellowWater => (5, 13, 1),
            Thing::Envelope => (5, 13, 1),
            _ => (1, i32::MAX, 0),
        }
    }
}

/// Whether the anomaly on a floor is the whole floor (the Showroom, the
/// Borrowed House: their own places, nothing put away in them); the
/// Stairwell's and the Loop's floors are rooms with their door
pub fn whole_floor_anomaly(scp: Option<&str>) -> bool {
    matches!(scp, Some("store" | "house"))
}

/// How many of the run's things a floor holds at most: one on the Lobby
/// (its emptiness is the point), two elsewhere
fn room_for(level: i32) -> usize {
    if level == 4 { 1 } else { 2 }
}

/// The run's things on `level` (at most two a floor, one on the Lobby), and
/// whether its hole is there: a function of the run's seed alone. On the
/// first descent (down to `RUN_FLOORS`) each thing lies on as many floors
/// as it says, none in the Showroom or the Borrowed House; below it, a
/// floor now and then has one.
pub fn run_things(run_seed: u32, level: i32) -> (Vec<Thing>, bool) {
    let ordinary = |n: i32| !whole_floor_anomaly(crate::level::scp::scp_for_level(run_seed, n));
    if level > RUN_FLOORS {
        let mut r = SplitMix64::new(run_seed, RUN_SALT ^ (level as u64).wrapping_mul(0x9E37_79B9));
        // (not what only the Building's floors hold: the ration, the salt in its cells)
        let things: Vec<Thing> = SPECIAL.iter().copied().filter(|t| t.floors().1 > 6).collect();
        let one = (r.next() < 0.6).then(|| things[((r.next() * things.len() as f64) as usize).min(things.len() - 1)]);
        let hole = r.next() < 0.25;
        return (one.into_iter().collect(), hole);
    }
    let mut r = SplitMix64::new(run_seed, RUN_SALT);
    let mut on: Vec<Vec<Thing>> = vec![vec![]; RUN_FLOORS as usize + 1];
    for t in SPECIAL {
        let (lo, hi, n) = t.floors();
        // (on the Lobby only what grows there: its emptiness is the point)
        let mut pool: Vec<i32> = (lo..=hi.min(RUN_FLOORS)).filter(|&l| ordinary(l) && (l != 4 || t == Thing::PaleMushrooms)).collect();
        for _ in 0..n {
            // the least crowded floors first, among them by the stream
            let fewest = pool.iter().map(|&l| on[l as usize].len()).min().unwrap_or(2);
            let open: Vec<i32> = pool.iter().copied().filter(|&l| on[l as usize].len() == fewest && fewest < room_for(l)).collect();
            if open.is_empty() {
                break;
            }
            let l = open[((r.next() * open.len() as f64) as usize).min(open.len() - 1)];
            on[l as usize].push(t);
            pool.retain(|&p| p != l);
        }
    }
    // the hole: three floors of the first descent
    let mut floors: Vec<i32> = (2..=RUN_FLOORS).filter(|&l| ordinary(l) && l != 4).collect();
    let mut holes = vec![];
    for _ in 0..3 {
        if floors.is_empty() {
            break;
        }
        let i = ((r.next() * floors.len() as f64) as usize).min(floors.len() - 1);
        holes.push(floors.remove(i));
    }
    let things = on.get(level.max(0) as usize).cloned().unwrap_or_default();
    (things, holes.contains(&level))
}

/// One place where something is put away, as planned: pure data.
#[derive(Clone, Debug, PartialEq)]
pub struct SecretPlan {
    pub kind: PlaceKind,
    pub thing: Thing,
    pub tile: (i32, i32),
    /// Where it is: on the floor, on a wall's face (`normal` out of it), or
    /// in the ceiling; `y` up from the floor
    pub x: f64,
    pub z: f64,
    pub y: f64,
    pub normal: (f64, f64),
    /// The cabinet and drawer it is under (`Drawer`)
    pub drawer: Option<(usize, usize)>,
    /// The scratched sign near it (x, z, y, normal), if it is signed, and
    /// whether the sign is crossed out (opened, nothing left)
    pub sign: Option<(f64, f64, f64, (f64, f64))>,
    pub crossed: bool,
}

/// A place on the level, and what has become of it.
#[derive(Clone, Debug)]
pub struct Secret {
    pub plan: SecretPlan,
    /// Its key in the run (`secrets_taken`): the level and its place in the plan
    pub key: String,
    pub opened: bool,
    /// What was in it is gone (taken, eaten, or there was nothing)
    pub emptied: bool,
    /// How many times it was used (the dish's sweets)
    pub uses: u32,
    /// What it is made of (the place, the thing in it, its sign)
    pub place: Option<Obj>,
    pub thing: Option<Obj>,
    pub sign: Option<Obj>,
    /// Moving parts of the place (the tile, the grille, the rag) and where
    /// they go when it is opened
    pub lid: Option<Obj>,
}

/// The secrets of the level being played
#[derive(Clone, Debug, Default)]
pub struct SecretsLevel {
    pub list: Vec<Secret>,
}

impl Game {
    /// Whether tile (x, z) is off the way: two tiles or more from her walk,
    /// from a trail that is not hers and from the shortest way, well away
    /// from the arrival, the door and her camp, and reached from the arrival.
    fn off_the_way(&self, ds: &[i32], way: &[bool], near_walk: &[bool], corner: Option<(i32, i32)>, x: i32, z: i32) -> bool {
        let i = self.t_idx(x, z);
        if self.is_wall_tile(x, z) || ds[i] < 0 || way[i] || near_walk[i] {
            return false;
        }
        let spawn = (self.tx(self.spawn_pos.x), self.tz(self.spawn_pos.z));
        let far = |t: (i32, i32), n: i32| (t.0 - x).abs() + (t.1 - z).abs() > n;
        let exit = self.exit_data.as_ref().map(|e| (self.tx(e.pos.x), self.tz(e.pos.z)));
        let camp = self.story.camp.as_ref().map(|c| c.tile);
        // and clear of the corner the one in the corner will stand in
        far(spawn, 3) && exit.is_none_or(|e| far(e, 3)) && camp.is_none_or(|c| far(c, 2)) && corner.is_none_or(|c| far(c, 2))
    }

    /// The level's secrets (`run_things` for what lies here, the hiding
    /// places off the way, signs, the hole), pure: from the level as built
    /// and the secrets' own stream. Empty where there is no way to the door,
    /// on an anomaly's floor, below none.
    pub fn plan_secrets(&self) -> Vec<SecretPlan> {
        let mut out: Vec<SecretPlan> = vec![];
        if whole_floor_anomaly(self.level_scp.as_deref()) || self.in_loop || self.in_stairwell || self.has_flag("nosecrets") {
            return out;
        }
        let Some(e) = self.exit_data.as_ref() else { return out };
        let exit = (self.tx(e.pos.x), self.tz(e.pos.z));
        let spawn = (self.tx(self.spawn_pos.x), self.tz(self.spawn_pos.z));
        let ds = self.story_walk_dist(spawn);
        let de = self.story_walk_dist(exit);
        if ds.get(self.t_idx(exit.0, exit.1)).is_none_or(|&d| d < 0) {
            return out;
        }
        let way = crate::entities::corner::main_way(self, &de, spawn.0, spawn.1);
        // within a tile of her walk, or of a trail that is not hers
        let mut near_walk = vec![false; (self.w * self.h) as usize];
        let walks = self.story.walk.iter().chain(self.story.false_trails.iter().flat_map(|f| f.walk.iter()));
        for &(x, z) in walks {
            for dz in -1..=1 {
                for dx in -1..=1 {
                    if self.in_grid(x + dx, z + dz) {
                        near_walk[self.t_idx(x + dx, z + dz)] = true;
                    }
                }
            }
        }
        let mut r = SplitMix64::new(self.current_seed, SECRET_SALT);
        let (things, hole) = run_things(self.run_seed, self.level_num);
        // how many places this floor has: two or three on the Building's,
        // on the Lobby only what grows there, two to four below
        let n = match self.level_num {
            ..=3 => 2 + (r.next() < 0.5) as usize,
            // the Lobby holds only what grows there (its emptiness is the point)
            4 => 0,
            _ => 2 + (r.next() * 3.0) as usize,
        }
        .max(things.len() + hole as usize);
        let corner = crate::entities::corner::corner_spot(self);
        let mut tiles: Vec<(i32, i32)> = vec![];
        for z in 0..self.h {
            for x in 0..self.w {
                if self.off_the_way(&ds, &way, &near_walk, corner, x, z) {
                    tiles.push((x, z));
                }
            }
        }
        // in an order of their own
        for i in (1..tiles.len()).rev() {
            tiles.swap(i, ((r.next() * (i + 1) as f64) as usize).min(i));
        }
        let apart = |out: &[SecretPlan], t: (i32, i32)| out.iter().all(|s| (s.tile.0 - t.0).abs() + (s.tile.1 - t.1).abs() >= 4);
        // what this floor holds: its things, its hole, then what any floor holds
        let mut wants: Vec<(Thing, &'static [PlaceKind])> = things.iter().map(|&t| (t, t.places())).collect();
        if hole {
            wants.push((Thing::Nothing, &[PlaceKind::Hole]));
        }
        while wants.len() < n {
            let u = r.next();
            // (no coin: you carry one at a time, from the machines' coin returns)
            let t = match u {
                u if u < 0.35 => Thing::Almond,
                u if u < 0.65 => Thing::Battery,
                u if u < 0.8 => Thing::Paper,
                _ => Thing::Nothing,
            };
            wants.push((t, Thing::Nothing.places()));
        }
        for (thing, kinds) in wants {
            // its kinds of place in turn (what any floor holds: from one of
            // them chosen by the stream), the first tile where one fits
            let start =
                if thing.special() || kinds.len() < 2 { 0 } else { ((r.next() * kinds.len() as f64) as usize).min(kinds.len() - 1) };
            let kinds: Vec<PlaceKind> = kinds[start..].iter().chain(kinds[..start].iter()).copied().collect();
            let mut placed = None;
            'kinds: for &kind in &kinds {
                for &t in &tiles {
                    if !apart(&out, t) {
                        continue;
                    }
                    let mut rr = r;
                    if let Some(p) = self.place_at(kind, thing, t, &mut rr) {
                        r = rr;
                        placed = Some(p);
                        break 'kinds;
                    }
                }
            }
            if let Some(p) = placed {
                out.push(p);
            }
        }
        // the sign: about half of them, on a wall near; the first a run
        // shows (on its first floor, at the first place there that is
        // opened, holds what any floor holds and has a wall in sight for it)
        // crossed out, the place already opened and empty: the sign taught
        // safely
        let mut lesson = self.level_num == 1;
        for p in out.iter_mut() {
            if !p.kind.closed() || p.kind == PlaceKind::Hole {
                continue;
            }
            let teach = lesson && !p.thing.special();
            if teach || r.next() < 0.5 {
                p.sign = self.sign_near(p.tile, &mut r);
                if teach && p.sign.is_some() {
                    p.thing = Thing::Nothing;
                    p.crossed = true;
                    lesson = false;
                }
            }
        }
        out
    }
}
