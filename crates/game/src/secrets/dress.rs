//! The hiding places made, their verbs, and what happens when one is
//! opened or a thing is taken.
use super::keeps::{place_words, thing_words, Took};
use super::{PlaceKind, Secret, SecretPlan, SecretsLevel, Thing};
use crate::prelude::*;

impl Game {
    /// Dress the level with its secrets (`plan_secrets`). Called by the
    /// level build after her trail, so the props round them make room.
    pub fn place_secrets(&mut self) {
        self.secrets = SecretsLevel::default();
        let plans = self.plan_secrets();
        let mut r = SplitMix64::new(self.current_seed, super::SECRET_SALT ^ 0xD5E5);
        for (i, p) in plans.into_iter().enumerate() {
            let s = self.dress_secret(i, p, &mut r);
            self.secrets.list.push(s);
        }
    }

    /// One place made: its parts, its tell, what is in it (shown now if it
    /// lies in the open), its sign, and its verbs.
    fn dress_secret(&mut self, i: usize, p: SecretPlan, r: &mut SplitMix64) -> Secret {
        let key = format!("{}:{}", self.level_num, i);
        let fy = self.floor_y_of(p.tile.0, p.tile.1);
        let face = p.normal.0.atan2(p.normal.1);
        let mut s =
            Secret { plan: p.clone(), key, opened: false, emptied: false, uses: 0, place: None, thing: None, sign: None, lid: None };
        let place = group();
        match p.kind {
            PlaceKind::Tile => {
                // the tile let down out of its grid, hanging from the edge still
                // in it, and the dark of the void over the ceiling where it
                // was (drawn just under the ceiling, which would hide it;
                // unlit, as a fallen tile's gap: no lit surface is black)
                let void = mesh(
                    plane_geo!(TILE - 0.02, TILE - 0.02),
                    basic_mat!({ color: 0x050504, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 }),
                );
                void.rotation().x = PI / 2.0;
                void.position().set(p.x, p.y - 0.003, p.z);
                place.add(&void);
                let tile = mesh(pg("FallenCeilingTiles__tile", box_geo!(TILE - 0.02, 0.016, TILE - 0.02)), self.secret_tile_mat());
                let (half, a) = ((TILE - 0.02) / 2.0, TILE_HANG);
                tile.position().set(p.x + half - half * a.cos(), p.y - 0.012 - half * a.sin(), p.z);
                tile.rotation().z = a;
                place.add(&tile);
                s.lid = Some(tile);
                // a chair under it (someone stood there)
                let chair = plain_chair();
                chair.position().set(p.x + r.range(-0.15, 0.15), fy, p.z + r.range(-0.15, 0.15));
                chair.rotation().y = r.range(0.0, PI * 2.0);
                place.add(&chair);
            }
            PlaceKind::Vent => {
                let inset = mesh(box_geo!(0.32, 0.18, 0.006), basic_mat!({ color: 0x0a0a09 }));
                inset.position().set(p.x - p.normal.0 * 0.0, fy + p.y, p.z - p.normal.1 * 0.0);
                inset.rotation().y = face;
                place.add(&inset);
                // the grille, hanging by the two screws left in it
                let grille = group();
                let metal = lambert!({ color: 0x8a8a84 });
                let g = mesh(pg("WallVents__grille", box_geo!(0.34, 0.2, 0.02)), &metal);
                g.position().set(0.17, -0.1, 0.0);
                grille.add(&g);
                for k in [0.0, 1.0] {
                    let screw = mesh(cyl_geo!(0.005, 0.005, 0.006, 6), lambert!({ color: 0x4a4a46 }));
                    screw.rotation().x = PI / 2.0;
                    screw.position().set(0.02, -0.02 - k * 0.16, 0.012);
                    grille.add(&screw);
                }
                let (ax, az) = (face.cos(), -face.sin());
                grille.position().set(p.x - ax * 0.17 + p.normal.0 * 0.012, fy + p.y + 0.1, p.z - az * 0.17 + p.normal.1 * 0.012);
                grille.rotation().y = face;
                grille.rotation().z = -0.09;
                place.add(&grille);
                s.lid = Some(grille);
            }
            PlaceKind::Radiator => {
                let iron = lambert!({ color: 0x5f5a52 });
                let rad = group();
                let body = mesh(pg("Radiators__body", box_geo!(0.7, 0.5, 0.1)), &iron);
                body.position().y = 0.35;
                rad.add(&body);
                for f in 0..8 {
                    let fin = mesh(pg("Radiators__fin", box_geo!(0.06, 0.48, 0.02)), &iron);
                    fin.position().set(-0.31 + f as f64 * 0.085, 0.35, 0.06);
                    rad.add(&fin);
                }
                let cap = mesh(pg("Radiators__cap", box_geo!(0.72, 0.03, 0.12)), &iron);
                cap.position().y = 0.615;
                rad.add(&cap);
                rad.position().set(p.x + p.normal.0 * 0.07, fy, p.z + p.normal.1 * 0.07);
                rad.rotation().y = face;
                place.add(&rad);
                // the rag stuffed down behind its fins
                let rag = mesh(sphere_geo!(0.05, 8, 6), lambert!({ color: 0x5a5448 }));
                rag.scale().set(1.4, 0.8, 0.5);
                let (ax, az) = (face.cos(), -face.sin());
                rag.position().set(p.x + p.normal.0 * 0.02 + ax * 0.24, fy + 0.62, p.z + p.normal.1 * 0.02 + az * 0.24);
                rag.rotation().y = face;
                place.add(&rag);
                s.lid = Some(rag);
            }
            PlaceKind::Hole => {
                let hole = mesh(plane_geo!(0.2, 0.2), hole_mat());
                hole.position().set(p.x, fy + p.y, p.z);
                hole.rotation().y = face;
                place.add(&hole);
                // the plaster it left on the floor
                let grit = lambert!({ color: if self.ground.is_empty() { 0xd6d0c0 } else { 0x8a857c } });
                for k in 0..6 {
                    let b = mesh(box_geo!(0.02, 0.008, 0.015), &grit);
                    let (a, d) = (k as f64 * 2.3 + r.next(), 0.05 + r.next() * 0.15);
                    b.position().set(
                        p.x + p.normal.0 * (0.08 + d) + a.cos() * 0.06,
                        fy + 0.004,
                        p.z + p.normal.1 * (0.08 + d) + a.sin() * 0.06,
                    );
                    b.rotation().y = a;
                    place.add(&b);
                }
            }
            PlaceKind::Hook => {
                let hook = group();
                let board = mesh(pg("CoatHooks__board", box_geo!(0.12, 0.06, 0.02)), lambert!({ color: 0x4a3a2a }));
                board.position().z = 0.01;
                hook.add(&board);
                let post = mesh(pg("CoatHooks__post", cyl_geo!(0.006, 0.006, 0.08, 6)), &self.mats.metal);
                post.rotation().x = PI / 2.0 - 0.4;
                post.position().set(0.0, 0.01, 0.05);
                hook.add(&post);
                hook.position().set(p.x, fy + p.y, p.z);
                hook.rotation().y = face;
                place.add(&hook);
            }
            PlaceKind::Table => {
                let wood = lambert!({ color: 0x4a3a28 });
                let table = group();
                let top = mesh(pg("HallSideTables__top", box_geo!(0.6, 0.03, 0.38)), &wood);
                top.position().y = 0.72;
                table.add(&top);
                for (lx, lz) in [(-0.27, -0.16), (0.27, -0.16), (-0.27, 0.16), (0.27, 0.16)] {
                    let leg = mesh(pg("HallSideTables__leg", box_geo!(0.03, 0.71, 0.03)), &wood);
                    leg.position().set(lx, 0.355, lz);
                    table.add(&leg);
                }
                table.position().set(p.x + p.normal.0 * 0.22, fy, p.z + p.normal.1 * 0.22);
                table.rotation().y = face;
                place.add(&table);
            }
            PlaceKind::Drawer => {
                // the drawer that will not quite shut
                if let Some((k, d)) = p.drawer {
                    if let Some(dr) = self.drawer_props.get(k).and_then(|c| c.drawers.get(d)) {
                        let mut dr = dr.borrow_mut();
                        dr.mesh.position().z += 0.035;
                        dr.content_type = format!("secret:{i}");
                    }
                }
            }
            PlaceKind::Cell => {
                // salt by its door, two small cones of it (the salt's own cell)
                if p.thing == Thing::Salt {
                    if let Some(c) = self.building.as_ref().and_then(|b| b.cells.iter().find(|c| c.tile == p.tile)) {
                        let d = crate::level::archgen::DIRS[c.door];
                        let (dx, dz) = (d[0] as f64, d[1] as f64);
                        let (cx, cz) = (self.wx(p.tile.0), self.wz(p.tile.1));
                        let salt = lambert!({ color: 0xe8e6e0 });
                        for sx in [-1.0, 1.0] {
                            let cone = mesh(cone_geo!(0.035, 0.06, 10), &salt);
                            let (x, z) = (cx + dx * (CELL / 2.0 + 0.25) - dz * sx * 0.85, cz + dz * (CELL / 2.0 + 0.25) + dx * sx * 0.85);
                            cone.position().set(x, self.floor_y_at(x, z) + 0.03, z);
                            place.add(&cone);
                        }
                    }
                }
            }
            PlaceKind::Corner => {}
        }
        place.user().gameplay = true;
        self.world.add(&place);
        s.place = Some(place);
        // the sign, a third hand's: a circle with a point in it, scratched in
        if let Some((x, z, y, n)) = p.sign {
            let sign = mesh(plane_geo!(0.08, 0.08), sign_mat(p.crossed));
            // (the floor in front of its wall: at the face itself it is the wall's)
            sign.position().set(x, self.floor_y_at(x + n.0 * 0.3, z + n.1 * 0.3) + y, z);
            sign.rotation().y = n.0.atan2(n.1);
            sign.user().gameplay = true;
            self.world.add(&sign);
            s.sign = Some(sign);
        }
        // what is in the open shows now; what is put away shows once found
        if !p.kind.closed() && p.thing.special() {
            s.thing = Some(self.show_thing(&p, r.next() * 7.0));
        }
        // the crossed-out place: found already, nothing left
        if p.crossed {
            s.opened = true;
            s.emptied = true;
            self.secret_open_pose(&s);
        }
        self.secret_verbs(i, &p);
        s
    }

    /// The thing put where it is seen: in the open where it lies, or where
    /// the hand that found it put it down (on the chair under the tile,
    /// before the vent, in the drawer, on the radiator)
    fn show_thing(&mut self, p: &SecretPlan, seed: f64) -> Obj {
        let o = self.make_thing(p.thing, seed);
        let fy = self.floor_y_of(p.tile.0, p.tile.1);
        let face = p.normal.0.atan2(p.normal.1);
        let (x, y, z) = match p.kind {
            PlaceKind::Tile => (p.x, fy + 0.53, p.z),
            PlaceKind::Vent => (p.x + p.normal.0 * 0.35, fy, p.z + p.normal.1 * 0.35),
            PlaceKind::Radiator => (p.x + p.normal.0 * 0.07, fy + 0.63, p.z + p.normal.1 * 0.07),
            PlaceKind::Table => (p.x + p.normal.0 * 0.22, fy + 0.735, p.z + p.normal.1 * 0.22),
            PlaceKind::Hook => (p.x + p.normal.0 * 0.07, fy + p.y - 0.13, p.z + p.normal.1 * 0.07),
            PlaceKind::Cell => (p.x, fy + p.y, p.z),
            PlaceKind::Drawer => {
                let c = p.drawer.and_then(|(k, _)| self.drawer_props.get(k)).map(|c| c.cabinet.clone());
                match c {
                    Some(c) => {
                        let cp = c.pos();
                        let yaw = c.rot().y;
                        (cp.x + yaw.sin() * 0.46, 0.28 + 0.45 - 0.13, cp.z + yaw.cos() * 0.46)
                    }
                    None => (p.x, fy, p.z),
                }
            }
            _ => (p.x, fy, p.z),
        };
        o.rotation().y = face + (seed * 3.1).sin() * 0.6;
        // a cap hangs by its brim, its crown out from the wall: tipped up in
        // a frame of its own, turned to the wall in its holder's
        let o = if p.kind == PlaceKind::Hook {
            o.rotation().y = 0.0;
            o.rotation().x = PI / 2.0 - 0.25;
            let holder = group();
            holder.add(&o);
            holder.rotation().y = face;
            holder
        } else {
            o
        };
        o.position().set(x, y, z);
        o.user().gameplay = true;
        self.world.add(&o);
        o
    }

    /// A place opened: the tile pushed aside, the grille swung down, the rag
    /// pulled out
    fn secret_open_pose(&self, s: &Secret) {
        let Some(lid) = &s.lid else { return };
        let p = &s.plan;
        match p.kind {
            PlaceKind::Tile => {
                // taken down and stood against the chair's legs on the floor
                let fy = self.floor_y_of(p.tile.0, p.tile.1);
                lid.position().set(p.x + 0.45, fy + 0.3, p.z);
                lid.rotation().z = 1.35;
            }
            PlaceKind::Vent => {
                lid.rotation().z = -1.45;
            }
            PlaceKind::Radiator => {
                let fy = self.floor_y_of(p.tile.0, p.tile.1);
                lid.position().set(p.x + p.normal.0 * 0.45, fy + 0.03, p.z + p.normal.1 * 0.45);
            }
            _ => {}
        }
    }

    /// Its verbs: the place's own (to open it), then the thing's (to take it)
    fn secret_verbs(&mut self, i: usize, p: &SecretPlan) {
        let fy = self.floor_y_of(p.tile.0, p.tile.1);
        let at = match p.kind {
            PlaceKind::Tile => v3(p.x, p.y - 0.2, p.z),
            PlaceKind::Drawer => v3(p.x, fy + 0.6, p.z),
            _ => v3(p.x + p.normal.0 * 0.2, fy + p.y.max(0.3), p.z + p.normal.1 * 0.2),
        };
        let verb = match p.kind {
            PlaceKind::Tile => Some("[E] LIFT THE TILE"),
            PlaceKind::Vent => Some("[E] PRISE THE GRILLE"),
            PlaceKind::Radiator => Some("[E] REACH BEHIND"),
            PlaceKind::Hole => Some("[E] REACH IN"),
            _ => None,
        };
        let open = move |g: &Game| g.secrets.list.get(i).is_some_and(|s| !s.opened);
        if let Some(v) = verb {
            let r = if p.kind == PlaceKind::Tile { 2.1 } else { 1.5 };
            self.interact(at, r, v, open, move |g| g.open_secret(i));
        }
        if p.thing.special() && p.kind != PlaceKind::Hole {
            let reach = if p.kind == PlaceKind::Tile { v3(p.x, fy + 0.6, p.z) } else { at };
            let take = move |g: &Game| g.secrets.list.get(i).is_some_and(|s| s.thing.is_some() && !s.emptied);
            self.interact(reach, 1.6, verb_of(p.thing), take, move |g| g.take_secret(i));
        }
    }

    /// Open place `i`: it shows what is in it, or gives it to you (what any
    /// floor holds goes into your pockets), or there was nothing. The hole
    /// gives back and takes (`hole_exchange`).
    pub fn open_secret(&mut self, i: usize) {
        let Some(s) = self.secrets.list.get(i) else { return };
        if s.opened {
            return;
        }
        let p = s.plan.clone();
        self.secrets.list[i].opened = true;
        let s = self.secrets.list[i].clone();
        self.secret_open_pose(&s);
        let fy = self.floor_y_of(p.tile.0, p.tile.1);
        self.play_from(p.x, fy + p.y, p.z, 3.0, 1.0, 0.6, move |g| g.sfx_secret_open(p.kind));
        if p.kind == PlaceKind::Hole {
            self.hole_exchange(i);
            return;
        }
        if p.thing.special() {
            let o = self.show_thing(&p, i as f64 * 1.7);
            self.secrets.list[i].thing = Some(o);
            return;
        }
        // what any floor holds, straight into your pockets
        let line = match p.thing {
            Thing::Almond => {
                self.player.bottles += 1;
                "ALMOND WATER, STILL SEALED"
            }
            Thing::Battery => {
                self.player.batteries += 1;
                "A BATTERY"
            }
            // (one coin at a time: none is put away, but never a second)
            Thing::Penny if self.player.coins == 0 => {
                self.player.coins += 1;
                "A COIN"
            }
            Thing::Paper => {
                self.player.papers += 3;
                "A FEW SHEETS OF PAPER"
            }
            _ => "NOTHING. ONLY DUST",
        };
        self.show_msg(line, 1.8);
        self.secret_emptied(i, None);
    }

    /// Take the thing in place `i` (eat it, drink it, put it on, keep it):
    /// it is gone from where it was, the notebook keeps what and where, and
    /// what it does stays with you (`keeps`). The dish keeps its sweets
    /// until the third.
    pub fn take_secret(&mut self, i: usize) {
        let Some(s) = self.secrets.list.get(i).cloned() else { return };
        if s.emptied || s.thing.is_none() {
            return;
        }
        let t = s.plan.thing;
        self.secrets.list[i].uses += 1;
        let uses = self.secrets.list[i].uses;
        let line = taste_of(t, uses);
        self.show_msg(line, 2.6);
        let p = s.plan.clone();
        self.play_from(p.x, self.floor_y_at(p.x, p.z) + 0.5, p.z, 2.0, 1.0, 0.6, move |g| g.sfx_take_thing(t));
        // a sweet from the dish: the dish stays until it is three
        let gone = t != Thing::Sweets || uses >= 3;
        if gone {
            if let Some(o) = &s.thing {
                o.set_visible(false);
            }
        }
        self.took_thing(t, &p, uses);
        if gone {
            self.secret_emptied(i, Some(t));
        } else {
            self.secrets_taken.insert(s.key.clone(), uses);
            self.save_game();
        }
    }

    /// What was in place `i` is gone for the rest of the run (a retry, a
    /// Continue): it stays so (`fit_secrets`)
    pub(crate) fn secret_emptied(&mut self, i: usize, _t: Option<Thing>) {
        let Some(s) = self.secrets.list.get_mut(i) else { return };
        s.emptied = true;
        let (key, uses) = (s.key.clone(), s.uses.max(1));
        self.secrets_taken.insert(key, uses);
        self.save_game();
    }

    /// Fit the secrets of a level just built to the run: what was taken
    /// from a place (or found empty) is gone, the place found open; the
    /// dish has fewer sweets.
    pub fn fit_secrets(&mut self) {
        for i in 0..self.secrets.list.len() {
            let key = self.secrets.list[i].key.clone();
            let Some(&uses) = self.secrets_taken.get(&key) else { continue };
            let s = &mut self.secrets.list[i];
            s.uses = uses;
            let dish = s.plan.thing == Thing::Sweets && uses < 3;
            if dish {
                continue;
            }
            s.opened = true;
            s.emptied = true;
            if let Some(o) = &s.thing {
                o.set_visible(false);
            }
            let s = s.clone();
            self.secret_open_pose(&s);
        }
    }
}

impl Game {
    /// A ceiling tile's face, as the fallen ones are (mineral fibre, aged)
    fn secret_tile_mat(&self) -> Mat {
        lambert!({ color: 0xbab29e })
    }

    /// The hole in the wall (after the first SCP-1162): reach in and your
    /// fingers find what you last used up (another bottle like the one you
    /// drank, another battery like the one you changed), and something you
    /// carry is gone, one thing among what you have. With nothing on you it
    /// takes from you instead (`keeps`: the stamina ceiling lower).
    pub(crate) fn hole_exchange(&mut self, i: usize) {
        let mut r = SplitMix64::new(self.current_seed, super::SECRET_SALT ^ 0x401E ^ i as u64);
        let p = &self.player;
        let mut have = vec![];
        if p.bottles > 0 {
            have.push(Thing::Almond);
        }
        if p.batteries > 0 {
            have.push(Thing::Battery);
        }
        if p.papers > 0 {
            have.push(Thing::Paper);
        }
        if p.coins > 0 {
            have.push(Thing::Penny);
        }
        let gives = self.last_used;
        let place = self.secrets.list[i].plan.clone();
        if have.is_empty() {
            self.took.push(Took { what: "hole_took".into(), level: self.level_num, place: place.kind.name().into() });
            self.refresh_body();
        } else {
            match have[((r.next() * have.len() as f64) as usize).min(have.len() - 1)] {
                Thing::Almond => self.player.bottles -= 1,
                Thing::Battery => self.player.batteries -= 1,
                Thing::Paper => self.player.papers -= self.player.papers.min(3),
                _ => self.player.coins -= 1,
            }
            self.took.push(Took { what: "hole".into(), level: self.level_num, place: place.kind.name().into() });
        }
        let found = match gives {
            Some(Thing::Almond) => {
                self.player.bottles += 1;
                "YOUR FINGERS FIND A BOTTLE"
            }
            Some(Thing::Battery) => {
                self.player.batteries += 1;
                "YOUR FINGERS FIND A BATTERY"
            }
            _ => "YOUR FINGERS FIND NOTHING",
        };
        // with nothing on you, whatever it gave, it takes from you: felt
        let line = if have.is_empty() { format!("{found}. SOMETHING TOUCHES YOUR WRIST") } else { found.to_string() };
        self.show_msg(&line, 2.6);
        self.secret_emptied(i, None);
    }

    /// You took `t` from `p` (the `uses`-th from it): the run keeps it
    /// (what it does: `keeps::body_of`); a yellowish almond water is almond
    /// water too
    pub(crate) fn took_thing(&mut self, t: Thing, p: &SecretPlan, uses: u32) {
        let what = if t == Thing::Sweets && uses >= 3 { "third_sweet" } else { t.id() };
        self.took.push(Took { what: what.into(), level: self.level_num, place: p.kind.name().into() });
        if t == Thing::YellowWater {
            self.player.stamina = (self.player.stamina + crate::player::DRINK_STAMINA * self.body.drink).min(self.body.stam_max);
        }
        // felt only as what it is (its taste or its weight: the line) and,
        // for your size, the change coming over you (`SIZE_EASE`); the rest
        // is learned by what follows (every sound and sway of the body is
        // some other thing's already: the dread's heart, the Watcher's ring)
        self.refresh_body();
    }

    /// What you are now, from what you took
    pub fn refresh_body(&mut self) {
        self.body = super::keeps::body_of(&self.took);
    }

    /// The notebook's page of what you took: the act, never the effect,
    /// one line a thing in the order taken, what and where
    pub fn what_i_took_page(&self) -> String {
        let mut lines = vec![];
        for t in &self.took {
            let place = |k: &str| {
                [
                    PlaceKind::Tile,
                    PlaceKind::Vent,
                    PlaceKind::Drawer,
                    PlaceKind::Radiator,
                    PlaceKind::Cell,
                    PlaceKind::Hole,
                    PlaceKind::Corner,
                    PlaceKind::Table,
                    PlaceKind::Hook,
                ]
                .into_iter()
                .find(|p| p.name() == k)
                .unwrap_or(PlaceKind::Corner)
            };
            let line = match t.what.as_str() {
                "hole" | "hole_took" => format!("I reached into a hole in a wall. Level {}.", t.level),
                "third_sweet" => format!("A third sweet from the dish. Level {}.", t.level),
                id => {
                    let Some(th) = Thing::from_id(id) else { continue };
                    format!("{}, {}. Level {}.", thing_words(th), place_words(place(&t.place), th), t.level)
                }
            };
            lines.push(line);
        }
        lines.join("\n")
    }
}

/// A ceiling tile's side, m (the grid's cell)
const TILE: f64 = CELL / 6.0;
/// How far a let-down ceiling tile hangs from its edge, radians (its free
/// edge some 14 cm down)
const TILE_HANG: f64 = 0.22;

/// The verb for a thing
pub fn verb_of(t: Thing) -> &'static str {
    match t {
        Thing::PaleMushrooms | Thing::BrownMushrooms => "[E] EAT THEM",
        Thing::Sweets => "[E] TAKE A SWEET",
        Thing::Seeds => "[E] EAT THEM",
        Thing::Bag | Thing::Cap | Thing::Coin => "[E] PUT IT ON",
        Thing::Salt => "[E] TAKE IT",
        Thing::Jelly | Thing::Ration => "[E] EAT IT",
        Thing::Medicine | Thing::YellowWater => "[E] DRINK IT",
        Thing::Envelope => "[E] PICK IT UP",
        _ => "[E] TAKE IT",
    }
}

/// What taking it is like: the thing and its taste or feel, never what it
/// does (the toast's words). The `uses`-th from the dish.
pub fn taste_of(t: Thing, uses: u32) -> &'static str {
    match t {
        Thing::PaleMushrooms => "YOU EAT THEM — SOFT, LIKE BREAD",
        Thing::BrownMushrooms => "YOU EAT THEM — DRY, AND BITTER",
        Thing::Sweets => match uses {
            1 => "A SWEET — SUGAR, THEN PENNIES",
            2 => "ANOTHER — THE SAME",
            _ => "ONE MORE — IT STICKS TO YOUR TEETH",
        },
        Thing::Seeds => "YOU EAT THEM — SHARP, AND SWEET",
        Thing::Bag => "YOU PUT IT ON — IT SMELLS OF SOMEONE ELSE",
        Thing::Salt => "YOU POCKET THE SALT",
        Thing::Jelly => "YOU EAT IT — IT TASTES OF NOTHING AT ALL",
        Thing::Ration => "YOU EAT IT — FLOURY, AND IT CRUMBLES",
        Thing::Medicine => "YOU DRINK IT — SYRUP, THEN WARM",
        Thing::Cap => "YOU PUT IT ON — IT FITS",
        Thing::Coin => "YOU PUT IT ON — IT TAPS AGAINST YOUR CHEST",
        Thing::YellowWater => "YOU DRINK — IT TASTES OF CASHEWS",
        Thing::Envelope => "YOU PICK IT UP — THERE IS HAIR IN IT",
        _ => "",
    }
}

/// The mark in a third hand: a small circle with a point in it, scratched
/// into the paint or the concrete (pale where the surface is broken);
/// crossed out, an X scored over it
fn sign_mat(crossed: bool) -> Mat {
    let tex = canvas_tex_opt(64, 64, false, move |ctx, w, h| {
        ctx.clear_rect(0, 0, w, h);
        let scratch = |ctx: &mut Canvas, pts: &[(f64, f64)]| {
            // a key's point dragged: a fine pale line, broken here and there, its edge darker
            for (col, wid) in [("rgba(40,36,30,0.35)", 3.4), ("rgba(226,220,204,0.8)", 1.6)] {
                ctx.set_stroke_style(col);
                ctx.set_line_width(wid);
                ctx.set_line_cap("round");
                ctx.begin_path();
                for (k, p) in pts.iter().enumerate() {
                    let (x, y) = (p.0 + (rnd() - 0.5) * 0.8, p.1 + (rnd() - 0.5) * 0.8);
                    if k == 0 || rnd() < 0.06 {
                        ctx.move_to(x, y);
                    } else {
                        ctx.line_to(x, y);
                    }
                }
                ctx.stroke();
            }
        };
        let circle: Vec<(f64, f64)> = (0..=28)
            .map(|k| {
                let a = k as f64 / 28.0 * PI * 2.0;
                (w / 2.0 + a.cos() * w * 0.3, h / 2.0 + a.sin() * h * 0.3)
            })
            .collect();
        scratch(ctx, &circle);
        // the point at its middle, dug in
        scratch(ctx, &[(w * 0.48, h * 0.48), (w * 0.53, h * 0.53), (w * 0.5, h * 0.52)]);
        if crossed {
            scratch(ctx, &[(w * 0.12, h * 0.12), (w * 0.88, h * 0.88)]);
            scratch(ctx, &[(w * 0.88, h * 0.12), (w * 0.12, h * 0.88)]);
        }
    });
    lambert!({ map: tex, transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 })
}

/// A fist-wide hole broken through a wall: dark inside, its rim of broken
/// plaster, cracks running off it
fn hole_mat() -> Mat {
    let tex = canvas_tex_opt(128, 128, false, |ctx, w, h| {
        ctx.clear_rect(0, 0, w, h);
        let (cx, cy) = (w / 2.0, h / 2.0);
        // cracks
        ctx.set_stroke_style("rgba(40,36,30,0.6)");
        ctx.set_line_width(1.2);
        for k in 0..6 {
            let a = k as f64 * 1.05 + rnd() * 0.4;
            ctx.begin_path();
            ctx.move_to(cx + a.cos() * 26.0, cy + a.sin() * 26.0);
            let mut p = (cx + a.cos() * 26.0, cy + a.sin() * 26.0);
            for _ in 0..4 {
                let b = a + (rnd() - 0.5) * 0.8;
                p = (p.0 + b.cos() * 8.0, p.1 + b.sin() * 8.0);
                ctx.line_to(p.0, p.1);
            }
            ctx.stroke();
        }
        // the broken rim, paler where the plaster shows
        ctx.set_fill_style("rgba(200,192,176,0.9)");
        ctx.begin_path();
        for k in 0..=28 {
            let a = k as f64 / 28.0 * PI * 2.0;
            let rr = 30.0 + (rnd() - 0.5) * 7.0;
            let (x, y) = (cx + a.cos() * rr, cy + a.sin() * rr);
            if k == 0 {
                ctx.move_to(x, y);
            } else {
                ctx.line_to(x, y);
            }
        }
        ctx.fill();
        // the dark inside
        ctx.set_fill_style("rgba(6,5,4,1)");
        ctx.begin_path();
        for k in 0..=28 {
            let a = k as f64 / 28.0 * PI * 2.0;
            let rr = 23.0 + (rnd() - 0.5) * 6.0;
            let (x, y) = (cx + a.cos() * rr, cy + a.sin() * rr);
            if k == 0 {
                ctx.move_to(x, y);
            } else {
                ctx.line_to(x, y);
            }
        }
        ctx.fill();
    });
    lambert!({ map: tex, transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 })
}

/// A plain wooden chair
fn plain_chair() -> Obj {
    let wood = lambert!({ color: 0x4c3b22 });
    let c = group();
    let seat = mesh(pg("Props__chair_seat", box_geo!(0.44, 0.04, 0.42)), &wood);
    seat.position().y = 0.46;
    c.add(&seat);
    for (x, z) in [(-0.19, -0.18), (0.19, -0.18), (-0.19, 0.18), (0.19, 0.18)] {
        let leg = mesh(pg("Props__chair_legTop", box_geo!(0.035, 0.44, 0.035)), &wood);
        leg.position().set(x, 0.22, z);
        c.add(&leg);
    }
    for s in [-1.0, 1.0] {
        let post = mesh(pg("Props__chair_post", box_geo!(0.035, 0.5, 0.035)), &wood);
        post.position().set(s * 0.19, 0.73, -0.19);
        c.add(&post);
    }
    let rail = mesh(pg("Props__chair_rail", box_geo!(0.42, 0.06, 0.025)), &wood);
    rail.position().set(0.0, 0.92, -0.19);
    c.add(&rail);
    c
}
