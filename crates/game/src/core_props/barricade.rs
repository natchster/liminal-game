//! The barricades (`docs/design/PROPS.md` 2): something people built across a
//! real opening to close it, chosen and built once the walls are final
//! (`barricades_settled`), on a stream of their own.
//!
//! An opening is a tile whose two sides across the way are walls and whose
//! two ends are open (a doorway in a partition, a corridor between solid
//! walls), measured on the solids: its width face to face, its top the
//! lintel or the ceiling. A barricade fills it wall to wall and floor to
//! near its top, and its slab (`Barricade`) stops a body and a look and
//! closes its tile to the creatures' ways and the place's way to the door.
//! The old choice (`tail_barricades`) still runs inside the core props for
//! its draws, so nothing placed after it moves; its tiles stand in for the
//! barricades while the themed props are placed, and are swapped for these
//! before her trail is laid.
use crate::assets::props::prop_part_as_modelled;
use crate::base::records::Barricade;
use crate::core_props::{PropsCtx, build_cardboard_stack, build_filing_cabinet};
use crate::level::archgen::{self, BOX_LINTEL, DIRS};
use crate::prelude::*;

/// The barricades' stream (`SplitMix64` with the level's seed)
pub const BARRICADE_SALT: u64 = 0xBA22_1CAD_E5A1_7F03;
/// An opening this wide, face to face, can be closed (m)
const MIN_W: f64 = 0.9;
const MAX_W: f64 = 4.1;
/// Never within this of where a floor begins (m)
const SPAWN_CLEAR: f64 = 6.0;
/// Two barricades at least this many tiles apart
const APART: i32 = 3;
/// What is left between its top and the lintel or ceiling (m)
const UNDER_TOP: f64 = 0.06;
/// Its sheets and braces stop this short of the faces (m)
const FACE_GAP: f64 = 0.012;

/// What a barricade is built of, by what the place was for
#[derive(Clone, Copy, Debug, PartialEq)]
enum Kind {
    /// sheets of plywood, braced across and bolted (halls, tunnels, plant rooms; anything wide)
    Sheets,
    /// boards nailed across a doorway, gaps between them
    Planks,
    /// an office's furniture: a desk on end, filing cabinets, chairs, boxes
    Office,
    /// a store room's: pallets on edge, a shelving unit, boxes
    Storage,
    /// a home's: a mattress on edge, a door off its hinges nailed across, a dresser
    Home,
}

/// An opening a barricade can close
#[derive(Clone, Copy, Debug)]
struct Opening {
    x: i32,
    z: i32,
    /// the way through runs along x (its faces are across z), else along z
    along_x: bool,
    /// the two faces across the way (world z if `along_x`, else x)
    lo: f64,
    hi: f64,
    /// the lintel's underside or the ceiling (m over the floor)
    top: f64,
}

impl Game {
    /// Choose the floor's barricades in its real openings and build them:
    /// as many as the old choice made (one at most on the Lobby), none on a
    /// floor with rules of its own, none in a pool hall; spawn to exit still
    /// solvable, every pickup still in reach, none near where the floor
    /// begins, none beside another. Then the build's barricade tiles become
    /// theirs.
    pub fn barricades_settled(&mut self) {
        let wanted = self.barricade_tiles.len();
        self.barricade_tiles.clear();
        if wanted == 0 || self.level_num < 4 || self.level_scp.is_some() {
            return;
        }
        let wanted = if self.level_num == 4 { wanted.min(1) } else { wanted };
        let Some(exit) = self.exit_data.as_ref().map(|e| (self.tx(e.pos.x), self.tz(e.pos.z))) else { return };
        let spawn = (self.tx(self.spawn_pos.x), self.tz(self.spawn_pos.z));
        let mut cands = self.openings();
        let mut r = SplitMix64::new(self.current_seed, BARRICADE_SALT);
        for i in (1..cands.len()).rev() {
            let j = (r.next() * (i + 1) as f64) as usize;
            cands.swap(i, j);
        }
        let pickups = self.pickup_tiles();
        let mut chosen: Vec<Opening> = vec![];
        for o in cands {
            if chosen.len() >= wanted {
                break;
            }
            if chosen.iter().any(|c| (c.x - o.x).abs().max((c.z - o.z).abs()) < APART) {
                continue;
            }
            let mut blocked: Vec<usize> = chosen.iter().map(|c| self.t_idx(c.x, c.z)).collect();
            blocked.push(self.t_idx(o.x, o.z));
            if !self.alt_path_exists(spawn.0, spawn.1, exit.0, exit.1, &blocked) {
                continue;
            }
            if !self.all_reached(spawn, &blocked, &pickups) {
                continue;
            }
            chosen.push(o);
        }
        for o in chosen {
            let seed = (self.current_seed as u64 ^ BARRICADE_SALT ^ (self.t_idx(o.x, o.z) as u64).wrapping_mul(0x9E37_79B9)) as u32;
            let kind = self.kind_for(&o);
            let (g, b) = crate::base::rng::with_own_stream(seed, || self.build_barricade(&o, kind));
            self.world.add(&g);
            self.barricade_tiles.push(b.tile);
            self.barricades.push(b);
        }
    }

    /// Every opening of the floor a barricade could close, in tile order
    fn openings(&self) -> Vec<Opening> {
        let Some(s) = self.solids.as_ref() else { return vec![] };
        let spawn = (self.spawn_pos.x, self.spawn_pos.z);
        let open = |x: i32, z: i32| self.in_grid(x, z) && !self.is_wall_tile(x, z);
        let mut out = vec![];
        for z in 1..self.h - 1 {
            for x in 1..self.w - 1 {
                if !open(x, z) {
                    continue;
                }
                let along_x = open(x + 1, z) && open(x - 1, z) && !open(x, z + 1) && !open(x, z - 1);
                let along_z = open(x, z + 1) && open(x, z - 1) && !open(x + 1, z) && !open(x - 1, z);
                if along_x == along_z {
                    continue;
                }
                let (wx, wz) = (self.wx(x), self.wz(z));
                if hypot2(wx - spawn.0, wz - spawn.1) < SPAWN_CLEAR {
                    continue;
                }
                if self.zone_purpose_at(x, z).is_some_and(|p| matches!(p.label, "pool hall" | "showroom")) {
                    continue;
                }
                // the faces across the way, at a knee, a hand and a head
                let (u, v) = (self.to_u(wx), self.to_v(wz));
                let (plus, minus) = if along_x { (dir_of(0, 1), dir_of(0, -1)) } else { (dir_of(1, 0), dir_of(-1, 0)) };
                let near = |d: usize| [0.3, 1.0, 1.8].iter().map(|&y| archgen::ray_axis(s, u, v, d, 2.0, y)).fold(f64::INFINITY, f64::min);
                let (tp, tm) = (near(plus), near(minus));
                if !tp.is_finite() || !tm.is_finite() {
                    continue;
                }
                let width = (tp + tm) * CELL;
                if !(MIN_W..=MAX_W).contains(&width) {
                    continue;
                }
                let mid = if along_x { wz } else { wx };
                let (lo, hi) = (mid - tm * CELL, mid + tp * CELL);
                let top = self.lintel_at(s, u, v).unwrap_or(f64::INFINITY).min(self.ceil_at_world(wx, wz));
                // nothing the floor needs in the way (a machine, a phone, a lift, the exit)
                if self.gameplay_near(wx, wz, along_x, lo, hi) {
                    continue;
                }
                out.push(Opening { x, z, along_x, lo, hi, top });
            }
        }
        out
    }

    /// The underside of a lintel over (u, v), if any (m)
    fn lintel_at(&self, s: &archgen::Solids, u: f64, v: f64) -> Option<f64> {
        let t = s.tile_of(u.floor(), v.floor())?;
        s.tile_items(t)
            .iter()
            .map(|&b| b as usize)
            .filter(|&b| s.kind(b) == BOX_LINTEL)
            .map(|b| s.box_at(b))
            .filter(|bx| u >= bx[0] as f64 && u <= bx[2] as f64 && v >= bx[1] as f64 && v <= bx[3] as f64)
            .map(|bx| bx[4] as f64)
            .reduce(f64::min)
    }

    /// Something the floor needs (its user `gameplay`: a machine, a phone,
    /// a lift, the exit, a pickup) within the opening's tile
    fn gameplay_near(&self, wx: f64, wz: f64, along_x: bool, lo: f64, hi: f64) -> bool {
        let (x0, x1, z0, z1) = if along_x { (wx - 1.0, wx + 1.0, lo, hi) } else { (lo, hi, wz - 1.0, wz + 1.0) };
        let near = |p: Vec3| p.x > x0 - 0.3 && p.x < x1 + 0.3 && p.z > z0 - 0.3 && p.z < z1 + 0.3;
        let pick = self.almonds.iter().chain(self.battery_items.iter()).any(|p| near(p.g.pos()))
            || self.papers.iter().any(|p| near(p.g.pos()))
            || self.lore_notes.iter().any(|n| near(n.g.pos()));
        pick || self.world.children().iter().any(|o| o.user().gameplay && near(o.pos()))
    }

    /// The tiles of what can be picked up (almond water, batteries, paper, notes)
    fn pickup_tiles(&self) -> Vec<(i32, i32)> {
        let at = |p: Vec3| (self.tx(p.x), self.tz(p.z));
        let mut out: Vec<(i32, i32)> = self.almonds.iter().chain(self.battery_items.iter()).map(|p| at(p.g.pos())).collect();
        out.extend(self.papers.iter().map(|p| at(p.g.pos())));
        out.extend(self.lore_notes.iter().map(|n| at(n.g.pos())));
        out
    }

    /// Is every tile of `want` reached from `from` with `blocked` closed?
    fn all_reached(&self, from: (i32, i32), blocked: &[usize], want: &[(i32, i32)]) -> bool {
        let mut seen = vec![false; (self.w * self.h).max(0) as usize];
        if !self.in_grid(from.0, from.1) || self.is_wall_tile(from.0, from.1) {
            return true;
        }
        let mut q = vec![from];
        seen[self.t_idx(from.0, from.1)] = true;
        let mut qi = 0;
        while qi < q.len() {
            let (x, z) = q[qi];
            qi += 1;
            for [dx, dz] in DIRS {
                let (nx, nz) = (x + dx, z + dz);
                if !self.in_grid(nx, nz) || self.is_wall_tile(nx, nz) {
                    continue;
                }
                let i = self.t_idx(nx, nz);
                if seen[i] || blocked.contains(&i) {
                    continue;
                }
                seen[i] = true;
                q.push((nx, nz));
            }
        }
        want.iter().all(|&(x, z)| !self.in_grid(x, z) || self.is_wall_tile(x, z) || seen[self.t_idx(x, z)])
    }

    /// What this opening's barricade is built of: the zone's purpose, else
    /// boards for a doorway and sheets for anything wider (the Lobby: boards
    /// or a sheet, nothing more)
    fn kind_for(&self, o: &Opening) -> Kind {
        let narrow = o.hi - o.lo <= 2.5;
        let plain = if narrow { Kind::Planks } else { Kind::Sheets };
        if self.level_num == 4 {
            return plain;
        }
        match self.zone_purpose_at(o.x, o.z).map(|p| p.label) {
            Some("office wing") => Kind::Office,
            Some("storage hall" | "maintenance") => Kind::Storage,
            Some("living quarters" | "parlour") => Kind::Home,
            _ => plain,
        }
    }

    /// The barricade across `o`, of `kind`: its group (stood in the opening,
    /// its x along the faces, z along the way) and its slab. Draws only from
    /// the stream it runs on.
    fn build_barricade(&mut self, o: &Opening, kind: Kind) -> (Obj, Barricade) {
        let span = o.hi - o.lo - 2.0 * FACE_GAP;
        let top = (o.top - UNDER_TOP).min(3.6);
        let g = group();
        let depth = match kind {
            Kind::Sheets => self.sheets(&g, span, top),
            Kind::Planks => self.planks(&g, span, top),
            Kind::Office => self.office_pile(&g, span, top),
            Kind::Storage => self.storage_pile(&g, span, top),
            Kind::Home => self.home_pile(&g, span, top),
        };
        let mid = (o.lo + o.hi) / 2.0;
        let (cx, cz) = if o.along_x { (self.wx(o.x), mid) } else { (mid, self.wz(o.z)) };
        g.position().set(cx, self.floor_y_at(cx, cz), cz);
        if o.along_x {
            g.rotation().y = -PI / 2.0;
        }
        {
            let mut u = g.user();
            u.placer = Some("place_props".into());
            u.s.insert("kind".into(), "barricades".into());
            u.gameplay = true;
        }
        // its slab: face to face across, as deep as it is (a body's breadth at least)
        let d = (depth / 2.0).max(0.15);
        let b = if o.along_x {
            Barricade { tile: self.t_idx(o.x, o.z), along_x: true, x0: cx - d, x1: cx + d, z0: o.lo, z1: o.hi, top }
        } else {
            Barricade { tile: self.t_idx(o.x, o.z), along_x: false, x0: o.lo, x1: o.hi, z0: cz - d, z1: cz + d, top }
        };
        (g, b)
    }

    /// Sheets of plywood cut to the opening on one face, three 2x4 braces
    /// across the other, bolted through; returns how deep it is
    fn sheets(&self, g: &Obj, span: f64, top: f64) -> f64 {
        let stud_m = stud_mat();
        let zinc = lambert!({ color: 0x55534e });
        let sheet = part("Barricade__plywood", 1.22, 2.44, 0.018);
        let stud = part("Barricade__stud", 2.44, 0.089, 0.038);
        let bolt = prop_part_as_modelled("Barricade__boltHead");
        let nut = prop_part_as_modelled("Barricade__nut");
        // which face the sheets are on (the other shows the braces)
        let face = if rnd() < 0.5 { 1.0 } else { -1.0 };
        let n = (span / 1.22).ceil().max(1.0);
        let sw = span / n;
        let sheet_h = top.min(2.44);
        for k in 0..n as usize {
            // each sheet its own (no two veneers alike), a hair narrower
            // than its share so the joints show, hung a few mm apart
            let m = mesh(sheet.clone(), plywood_mat());
            m.set_scale((sw - 0.006) / 1.22, sheet_h / 2.44, 1.0);
            let x = -span / 2.0 + sw * (k as f64 + 0.5);
            m.position().set(x, rand2(0.0, 0.015), face * (0.001 + 0.004 * (k % 2) as f64));
            m.rotation().y = if face < 0.0 { PI } else { 0.0 };
            m.rotation().z = rand2(-0.005, 0.005);
            g.add(&m);
        }
        // a strip ripped to fill what is over 2.44 m
        if top > sheet_h + 0.08 {
            let m = mesh(sheet.clone(), plywood_mat());
            m.set_scale(span / 1.22, (top - sheet_h - 0.01) / 2.44, 1.0);
            m.position().set(0.0, sheet_h + 0.005, face * 0.001);
            m.rotation().y = if face < 0.0 { PI } else { 0.0 };
            g.add(&m);
        }
        for frac in [0.25, 0.5, 0.75] {
            let y = sheet_h * frac + rand2(-0.05, 0.05);
            let b = mesh(stud.clone(), &stud_m);
            b.set_scale(span / 2.44, 1.0, 1.0);
            b.position().set(rand2(-0.01, 0.01), y - 0.0445, 0.0);
            b.rotation().y = if face > 0.0 { PI } else { 0.0 };
            b.rotation().z = rand2(-0.01, 0.01);
            g.add(&b);
            for bx in [-0.38, 0.0, 0.38] {
                let bx = bx * span + rand2(-0.03, 0.03);
                if let Some(h) = &bolt {
                    let m = mesh(h.clone(), &zinc);
                    m.position().set(bx, y, face * 0.02);
                    m.rotation().y = if face < 0.0 { PI } else { 0.0 };
                    g.add(&m);
                }
                if let Some(nn) = &nut {
                    let m = mesh(nn.clone(), &zinc);
                    m.position().set(bx, y, -face * 0.038);
                    m.rotation().y = if face > 0.0 { PI } else { 0.0 };
                    g.add(&m);
                }
            }
        }
        0.12
    }

    /// Boards nailed across a doorway onto a 2x4 stood against each face,
    /// from near the floor to the head, gaps between them, of mixed widths
    /// (1x4, 1x6, 1x8), one or two askew over the rest; a cut-off on the floor
    fn planks(&self, g: &Obj, span: f64, top: f64) -> f64 {
        let board = part("Barricade__board", 2.44, 0.14, 0.019);
        let stud = part("Barricade__stud", 2.44, 0.089, 0.038);
        let stud_m = stud_mat();
        let face = if rnd() < 0.5 { 1.0 } else { -1.0 };
        let reach = top.min(2.3);
        // the uprights the boards are nailed to, one at each face
        for side in [-1.0, 1.0] {
            let u = mesh(stud.clone(), &stud_m);
            u.set_scale(reach / 2.44, 1.0, 1.0);
            u.rotation().z = PI / 2.0;
            // stood up (its length about its middle): its 89 mm face across
            // the way, against the wall, from the floor
            u.position().set(side * (span / 2.0 - 0.045), reach / 2.0, -face * 0.038);
            g.add(&u);
        }
        let mut y = rand2(0.05, 0.14);
        let mut k = 0;
        while y < reach - 0.1 {
            let wide = choice(&[0.089, 0.14, 0.14, 0.184]);
            let m = mesh(board.clone(), board_mat());
            let askew = rnd() < 0.2;
            let tilt = if askew { rand2(-0.16, 0.16) } else { rand2(-0.025, 0.025) };
            // a board askew needs to be shorter to stay within the faces
            let len = span / tilt.cos().max(0.9) - 0.02;
            m.set_scale(len / 2.44, wide / 0.14, 1.0);
            // the askew ones over the rest
            m.position().set(rand2(-0.01, 0.01), y, face * (0.002 + if askew { 0.021 } else { 0.0 }));
            m.rotation().y = if face < 0.0 { PI } else { 0.0 };
            m.rotation().z = tilt;
            g.add(&m);
            y += wide + rand2(0.03, 0.22);
            k += 1;
        }
        let _ = k;
        // a cut-off left on the floor
        let m = mesh(board.clone(), board_mat());
        m.set_scale(rand2(0.12, 0.25), 1.0, 1.0);
        m.rotation().x = -PI / 2.0;
        m.rotation().z = rand2(-1.0, 1.0);
        m.position().set(rand2(-0.3, 0.3) * span, 0.005, face * rand2(0.2, 0.45));
        g.add(&m);
        0.1
    }

    /// An office's furniture shoved into the opening: a steel desk on its
    /// end, filing cabinets (one on its back at the foot), boxes in the
    /// gaps, chairs upside down on top; returns how deep it is
    fn office_pile(&mut self, g: &Obj, span: f64, top: f64) -> f64 {
        let paint = lambert!({ color: choice(&[0x7d7a70u32, 0x6b6f66, 0x8a8273]) });
        let laminate = lambert!({ color: 0x9a8e74 });
        let steel = lambert!({ color: 0x6a6a66 });
        let shell = lambert!({ color: choice(&[0x3c4250u32, 0x5a4a3a, 0x6e6a60]) });
        let desk = part("Barricade__desk", 1.52, 0.72, 0.76);
        let desk_top = part("Barricade__deskTop", 1.54, 0.04, 0.78);
        let frame = part("Barricade__chairFrame", 0.45, 0.8, 0.46);
        let seat = part("Barricade__chairShell", 0.44, 0.35, 0.45);
        let mut cx = PropsCtx::default();
        let mut x = -span / 2.0;
        let mut heights: Vec<(f64, f64, f64)> = vec![]; // (x0, x1, top of the column)
        // the desk on its end against one face: its length up, its top toward the way
        if span > 1.1 {
            let d = group();
            let side = if rnd() < 0.5 { 1.0 } else { -1.0 };
            for (geo, mat) in [(&desk, &paint), (&desk_top, &laminate)] {
                d.add(&mesh(geo.clone(), mat));
            }
            d.rotation().z = side * PI / 2.0;
            d.rotation().y = rand2(-0.06, 0.06);
            // stood on its end the desk is 0.75 across (its height), 1.52 up
            d.position().set(x + 0.38 + side * 0.0, 0.76, rand2(-0.05, 0.05));
            if side < 0.0 {
                d.position().x = x + 0.38;
            }
            g.add(&d);
            heights.push((x, x + 0.76, 1.52));
            x += 0.78;
        }
        // filing cabinets upright while there is room
        while x + 0.62 < span / 2.0 - 0.1 {
            let c = build_filing_cabinet(self, &mut cx);
            c.g.position().set(x + 0.31, 0.0, rand2(-0.08, 0.08));
            c.g.rotation().y = PI + rand2(-0.08, 0.08);
            c.g.rotation().z = rand2(-0.03, 0.03);
            g.add(&c.g);
            heights.push((x, x + 0.62, 1.42));
            x += 0.64;
        }
        // boxes in what is left
        if x < span / 2.0 - 0.3 {
            let b = build_cardboard_stack(self, &mut cx);
            b.g.position().set((x + span / 2.0) / 2.0, 0.0, rand2(-0.1, 0.1));
            g.add(&b.g);
            heights.push((x, span / 2.0, 0.9));
        }
        // a cabinet on its back at the foot, on the near side
        if rnd() < 0.6 {
            let c = build_filing_cabinet(self, &mut cx);
            c.g.rotation().x = -PI / 2.0;
            // on its back its depth (0.55, about its middle) is its height;
            // turned on the floor, not tipped
            c.g.position().y = 0.285;
            let turn = group();
            turn.add(&c.g);
            turn.rotation().y = rand2(-0.2, 0.2);
            turn.position().set(rand2(-0.3, 0.3) * span, 0.0, 0.85);
            g.add(&turn);
        }
        // chairs upside down on top, as high as they go
        for &(x0, x1, h) in &heights {
            if h + 0.8 > top - 0.05 || x1 - x0 < 0.5 {
                continue;
            }
            let c = group();
            c.add(&mesh(frame.clone(), &steel));
            c.add(&mesh(seat.clone(), &shell));
            c.rotation().x = PI;
            c.rotation().y = rand2(-0.6, 0.6);
            c.position().set((x0 + x1) / 2.0, h + 0.8, rand2(-0.1, 0.1));
            g.add(&c);
        }
        0.9
    }

    /// A store room's: pallets stood on edge across the opening, a shelving
    /// unit, boxes at the foot; returns how deep it is
    fn storage_pile(&mut self, g: &Obj, span: f64, top: f64) -> f64 {
        let pine = pallet_mat();
        let steel = lambert!({ color: 0x6e6e6a });
        let chip = lambert!({ color: 0x9c8a68 });
        let pallet = part("Barricade__pallet", 1.219, 0.137, 1.016);
        let shelf = part("Barricade__shelving", 0.91, 1.83, 0.46);
        let deck = part("Barricade__shelfDeck", 0.87, 0.016, 0.42);
        let mut cx = PropsCtx::default();
        let mut x = -span / 2.0;
        // the shelving unit upright against one face, its decks in it
        if span > 1.6 && top > 1.9 {
            let sg = group();
            sg.add(&mesh(shelf.clone(), &steel));
            for k in 0..5 {
                let d = mesh(deck.clone(), &chip);
                d.position().y = 0.06 + k as f64 * (1.83 - 0.12) / 4.0 + 0.018;
                sg.add(&d);
            }
            sg.position().set(x + 0.46, 0.0, rand2(-0.05, 0.05));
            sg.rotation().y = rand2(-0.05, 0.05);
            g.add(&sg);
            x += 0.93;
        }
        // pallets on edge, leaning, side by side, stacked two high where it is tall
        while x < span / 2.0 - 0.3 {
            let w = (span / 2.0 - x).min(1.219);
            for (row, y0) in [(0, 0.0), (1, 1.02)] {
                if row == 1 && top < 2.3 {
                    continue;
                }
                let p = mesh(pallet.clone(), &pine);
                // on edge: its length across, its width up, its deck facing the way
                p.rotation().x = PI / 2.0 + rand2(-0.12, 0.12);
                p.set_scale(w / 1.219, 1.0, 1.0);
                p.position().set(x + w / 2.0, y0 + 0.508, rand2(-0.08, 0.08));
                g.add(&p);
            }
            x += w + 0.01;
        }
        // boxes at the foot on one side
        let b = build_cardboard_stack(self, &mut cx);
        b.g.position().set(rand2(-0.3, 0.3) * span, 0.0, if rnd() < 0.5 { 0.6 } else { -0.6 });
        g.add(&b.g);
        let _ = top;
        0.5
    }

    /// A home's: a mattress stood on edge, a door off its hinges nailed
    /// across with two battens, a dresser on its back at the foot; returns
    /// how deep it is
    fn home_pile(&mut self, g: &Obj, span: f64, top: f64) -> f64 {
        let ticking = mattress_mat();
        let primer = lambert!({ color: 0xcfc8b6 });
        let veneer = lambert!({ color: 0x6b4a30 });
        let stud_m = stud_mat();
        let mattress = part("Barricade__mattress", 0.97, 0.2, 1.91);
        let door = part("Barricade__door", 0.81, 2.03, 0.035);
        let dresser = part("Barricade__dresser", 0.78, 0.82, 0.47);
        let stud = part("Barricade__stud", 2.44, 0.089, 0.038);
        let mut x = -span / 2.0;
        // mattresses on edge, leaning on the door's far side: two at most,
        // a wardrobe's worth of planks across what is left
        let mut beds = 0;
        while x < span / 2.0 - 0.5 && beds < 2 {
            beds += 1;
            let w = (span / 2.0 - x).min(0.97);
            let m = mesh(mattress.clone(), &ticking);
            // stood on its end: its length up (rotated about x, its length
            // runs from −L/2 to L/2 about its middle), its thickness along the way
            let k = (top.min(2.1) / 1.91).min(1.0);
            // leant back on what is behind it, turned a little so its edge shows
            let lean = rand2(0.08, 0.16);
            let mg = group();
            m.rotation().x = -PI / 2.0;
            m.set_scale(w / 0.97, 1.0, k);
            m.position().set(0.0, 1.91 * k / 2.0, 0.0);
            mg.add(&m);
            mg.rotation().x = -lean;
            mg.rotation().y = rand2(-0.25, 0.25);
            // (raised by what its lean dips its foot's back edge)
            mg.position().set(x + w / 2.0, 0.2 * lean.sin() + 0.003, -0.1 + rand2(-0.04, 0.04));
            g.add(&mg);
            x += w + 0.005;
        }
        if x < span / 2.0 - 0.3 {
            let wood = board_mat();
            let board = part("Barricade__board", 2.44, 0.14, 0.019);
            let w = span / 2.0 - x;
            let mut y = 0.2;
            while y < top.min(2.2) {
                let m = mesh(board.clone(), &wood);
                m.set_scale(w / 2.44, 1.0, 1.0);
                m.position().set(x + w / 2.0, y, -0.1);
                m.rotation().z = rand2(-0.04, 0.04);
                g.add(&m);
                y += 0.14 + rand2(0.08, 0.2);
            }
        }
        // the door across on the near side, nailed with two battens
        // the door propped across the front, its foot out, leaning on the pile
        let dg = group();
        let d = mesh(door.clone(), &primer);
        d.rotation().z = -PI / 2.0;
        d.position().set(-1.015, 0.405, 0.0);
        dg.add(&d);
        let lean = rand2(0.25, 0.4);
        let roll = rand2(-0.12, 0.12);
        dg.rotation().x = -lean;
        dg.rotation().z = roll;
        // (raised by what its lean and roll dip a corner)
        dg.position().set(rand2(-0.2, 0.2) * (span - 2.03).max(0.0), 0.02 * lean.sin() + 1.02 * roll.abs().sin() + 0.003, 0.35);
        g.add(&dg);
        for y in [0.6, 1.25] {
            let b = mesh(stud.clone(), &stud_m);
            b.set_scale(span / 2.44, 1.0, 1.0);
            b.position().set(0.0, y + rand2(-0.05, 0.05), 0.09);
            b.rotation().z = rand2(-0.04, 0.04);
            g.add(&b);
        }
        // a dresser shoved against the foot, its drawers out toward you
        let dr = mesh(dresser.clone(), &veneer);
        let side = if rnd() < 0.5 { 1.0 } else { -1.0 };
        dr.rotation().y = rand2(-0.25, 0.25) + if side < 0.0 { PI } else { 0.0 };
        dr.position().set(rand2(-0.25, 0.25) * span, 0.0, side * 0.62);
        g.add(&dr);
        let _ = top;
        0.6
    }
}

/// The barricade part `name` as modelled, or a box of its size
fn part(name: &str, w: f64, h: f64, d: f64) -> Geo {
    prop_part_as_modelled(name).unwrap_or_else(|| box_geo!(w, h, d))
}

/// A 1x6 board: rough-sawn pine, greyed, its grain along it
fn board_mat() -> Mat {
    let tex = canvas_tex_opt(512, 32, false, |ctx, w, h| {
        ctx.set_fill_style("#a08a68");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..24 {
            let y0 = rnd() * h;
            ctx.set_stroke_style(format!("rgba(90,70,45,{:.2})", 0.12 + rnd() * 0.2).as_str());
            ctx.set_line_width(0.5 + rnd());
            ctx.begin_path();
            ctx.move_to(0, y0);
            let mut x = 0.0;
            while x < w {
                x += 20.0;
                ctx.line_to(x, y0 + (x * 0.012 + y0).sin() * 1.5);
            }
            ctx.stroke();
        }
        for _ in 0..40 {
            ctx.set_fill_style(format!("rgba(60,55,48,{:.2})", 0.05 + rnd() * 0.12).as_str());
            ctx.fill_rect(rnd() * w, 0, 2.0 + rnd() * 20.0, h);
        }
        // nail heads near each end
        ctx.set_fill_style("rgba(40,36,32,0.8)");
        for x in [10.0, w - 12.0] {
            for y in [h * 0.3, h * 0.7] {
                ctx.begin_path();
                ctx.arc(x, y, 1.6, 0, 7);
                ctx.fill();
            }
        }
    });
    lambert!({ map: &tex })
}

/// A pallet's pine: rough, greyed, a stamp on a block
fn pallet_mat() -> Mat {
    let tex = canvas_tex_opt(256, 256, false, |ctx, w, h| {
        ctx.set_fill_style("#9a8566");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..50 {
            let x0 = rnd() * w;
            ctx.set_stroke_style(format!("rgba(80,62,40,{:.2})", 0.1 + rnd() * 0.2).as_str());
            ctx.set_line_width(0.6 + rnd() * 1.4);
            ctx.begin_path();
            ctx.move_to(x0, 0);
            ctx.line_to(x0 + rand2(-8.0, 8.0), h);
            ctx.stroke();
        }
        for _ in 0..30 {
            ctx.set_fill_style(format!("rgba(50,46,40,{:.2})", 0.06 + rnd() * 0.12).as_str());
            ctx.begin_path();
            ctx.arc(rnd() * w, rnd() * h, 2.0 + rnd() * 9.0, 0, 7);
            ctx.fill();
        }
    });
    lambert!({ map: &tex })
}

/// Mattress ticking: a yellowed off-white damask, quilted, with two or
/// three old stains, each a pale field inside a darker, uneven tide line
fn mattress_mat() -> Mat {
    let tex = canvas_tex_opt(256, 512, false, |ctx, w, h| {
        ctx.set_fill_style("#bdb39c");
        ctx.fill_rect(0, 0, w, h);
        // the damask: a faint woven figure, scrolls in rows, barely darker
        ctx.set_stroke_style("rgba(140,128,104,0.18)");
        ctx.set_line_width(1.0);
        let mut y = 10.0;
        let mut row = 0;
        while y < h {
            let mut x = if row % 2 == 0 { 8.0 } else { 21.0 };
            while x < w {
                ctx.begin_path();
                ctx.move_to(x - 7.0, y + 6.0);
                ctx.bezier_curve_to(x - 7.0, y - 8.0, x + 7.0, y - 8.0, x + 7.0, y + 6.0);
                ctx.stroke();
                x += 26.0;
            }
            y += 22.0;
            row += 1;
        }
        // the quilting's stitched lines
        ctx.set_stroke_style("rgba(110,100,80,0.25)");
        ctx.set_line_width(1.5);
        let mut y = 0.0;
        while y < h {
            ctx.begin_path();
            ctx.move_to(0, y);
            ctx.line_to(w, y + 6.0);
            ctx.stroke();
            y += 48.0;
        }
        // old stains: an uneven outline, a pale fill, a darker tide line
        for _ in 0..(2 + (rnd() * 2.0) as i32) {
            let (cx, cy) = (rand2(0.15, 0.85) * w, rand2(0.1, 0.9) * h);
            let r = rand2(18.0, 55.0);
            let n = 22;
            let pts: Vec<(f64, f64)> = (0..n)
                .map(|i| {
                    let a = i as f64 / n as f64 * std::f64::consts::TAU;
                    let rr = r * (0.7 + 0.45 * rnd()) * (1.0 + 0.25 * (a * 3.0).sin());
                    (cx + a.cos() * rr, cy + a.sin() * rr * rand2(0.6, 0.9))
                })
                .collect();
            ctx.begin_path();
            ctx.move_to(pts[0].0, pts[0].1);
            for i in 1..=n {
                let (x0, y0) = pts[(i - 1) % n];
                let (x1, y1) = pts[i % n];
                ctx.quadratic_curve_to(x0, y0, (x0 + x1) / 2.0, (y0 + y1) / 2.0);
            }
            ctx.close_path();
            ctx.set_fill_style("rgba(150,120,72,0.13)");
            ctx.fill();
            ctx.set_stroke_style("rgba(118,88,48,0.32)");
            ctx.set_line_width(1.5 + rnd() * 1.5);
            ctx.stroke();
        }
        // grime where hands and the floor have been
        let gr = ctx.create_linear_gradient(0, 0, 0, h);
        gr.add_color_stop(0, "rgba(70,60,45,0.18)");
        gr.add_color_stop(0.5, "rgba(70,60,45,0.0)");
        gr.add_color_stop(1, "rgba(60,50,38,0.3)");
        ctx.set_fill_style(gr);
        ctx.fill_rect(0, 0, w, h);
    });
    lambert!({ map: &tex })
}

/// The `DIRS` index of (dx, dz)
fn dir_of(dx: i32, dz: i32) -> usize {
    DIRS.iter().position(|d| d[0] == dx && d[1] == dz).unwrap_or(0)
}

/// Sheathing plywood: a fir face veneer, its figure the cathedral arches a
/// rotary-cut log leaves and the fine grain between them, a football
/// patch or two, greyed by years and darker toward the floor
fn plywood_mat() -> Mat {
    let tex = canvas_tex_opt(256, 512, false, |ctx, w, h| {
        ctx.set_fill_style("#9c8564");
        ctx.fill_rect(0, 0, w, h);
        // fine straight grain
        for _ in 0..120 {
            let x = rnd() * w;
            ctx.set_stroke_style(format!("rgba(90,68,42,{:.2})", 0.04 + rnd() * 0.08).as_str());
            ctx.set_line_width(0.5 + rnd());
            ctx.begin_path();
            ctx.move_to(x, 0);
            ctx.line_to(x + rand2(-3.0, 3.0), h);
            ctx.stroke();
        }
        // the cathedral figure: nested arches, each a band of latewood
        for _ in 0..1 {
            let cx = rand2(0.2, 0.8) * w;
            let top = rand2(0.05, 0.6) * h;
            let n = 6 + (rnd() * 5.0) as i32;
            for k in 0..n {
                let half = 10.0 + k as f64 * rand2(9.0, 13.0);
                let y0 = top + k as f64 * rand2(14.0, 22.0);
                ctx.set_stroke_style(format!("rgba(105,78,48,{:.2})", 0.12 + rnd() * 0.12).as_str());
                ctx.set_line_width(1.5 + rnd() * 2.5);
                ctx.begin_path();
                ctx.move_to(cx - half, h);
                ctx.bezier_curve_to(cx - half, y0 + 60.0, cx - half * 0.4, y0, cx, y0);
                ctx.bezier_curve_to(cx + half * 0.4, y0, cx + half, y0 + 60.0, cx + half, h);
                ctx.stroke();
            }
        }
        // a football patch where a knot was cut out
        for _ in 0..2 {
            let (x, y) = (rnd() * w, rnd() * h);
            ctx.set_fill_style("rgba(170,145,105,0.5)");
            ctx.begin_path();
            ctx.ellipse(x, y, 7, 18, 0, 0, 7);
            ctx.fill();
            ctx.set_stroke_style("rgba(70,52,32,0.45)");
            ctx.set_line_width(1);
            ctx.stroke();
        }
        // years: greyed all over, darker and dirtier toward the floor, a tide line
        ctx.set_fill_style("rgba(85,82,74,0.28)");
        ctx.fill_rect(0, 0, w, h);
        let gr = ctx.create_linear_gradient(0, h * 0.6, 0, h);
        gr.add_color_stop(0, "rgba(50,42,32,0)");
        gr.add_color_stop(1, "rgba(45,38,28,0.55)");
        ctx.set_fill_style(gr);
        ctx.fill_rect(0, h * 0.6, w, h * 0.4);
        ctx.set_stroke_style("rgba(70,55,38,0.35)");
        ctx.set_line_width(2);
        ctx.begin_path();
        let ty = h - 40.0 - rnd() * 30.0;
        ctx.move_to(0, ty);
        let mut x = 0.0;
        while x < w {
            x += 8.0;
            ctx.line_to(x, ty + (x * 0.05).sin() * 5.0 + rnd() * 3.0);
        }
        ctx.stroke();
        for _ in 0..80 {
            ctx.set_fill_style(format!("rgba(40,34,26,{:.2})", 0.04 + rnd() * 0.1).as_str());
            ctx.begin_path();
            ctx.arc(rnd() * w, h * (0.4 + 0.6 * rnd()), 1.0 + rnd() * 4.0, 0, 7);
            ctx.fill();
        }
        // the nails along its edges, every foot or so, a little rust round each
        ctx.set_fill_style("rgba(55,45,38,0.75)");
        let nail = |ctx: &mut Canvas, x: f64, y: f64| {
            ctx.begin_path();
            ctx.arc(x + rand2(-1.0, 1.0), y + rand2(-1.0, 1.0), 1.4, 0, 7);
            ctx.fill();
        };
        let mut t = 14.0;
        while t < h - 8.0 {
            nail(ctx, 6.0, t);
            nail(ctx, w - 6.0, t);
            t += 26.0 + rand2(-3.0, 3.0);
        }
        let mut t = 14.0;
        while t < w - 8.0 {
            nail(ctx, t, 6.0);
            nail(ctx, t, h - 6.0);
            t += 26.0 + rand2(-3.0, 3.0);
        }
    });
    lambert!({ map: &tex })
}

/// A 2x4: spruce-pine-fir, pale, its grain along it, a mill stamp
fn stud_mat() -> Mat {
    let tex = canvas_tex_opt(512, 64, false, |ctx, w, h| {
        ctx.set_fill_style("#c2a77c");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..30 {
            let y0 = rnd() * h;
            ctx.set_stroke_style(format!("rgba(120,90,55,{:.2})", 0.1 + rnd() * 0.2).as_str());
            ctx.set_line_width(0.5 + rnd() * 1.5);
            ctx.begin_path();
            ctx.move_to(0, y0);
            let mut x = 0.0;
            while x < w {
                x += 16.0;
                ctx.line_to(x, y0 + (x * 0.01 + y0).sin() * 2.0);
            }
            ctx.stroke();
        }
        ctx.set_fill_style("rgba(60,50,70,0.35)");
        ctx.set_font("bold 11px Helvetica Neue");
        ctx.fill_text("SPF  S-DRY  STUD", w * 0.55, h * 0.6);
        let gr = ctx.create_linear_gradient(0, 0, w, 0);
        gr.add_color_stop(0, "rgba(60,55,45,0.18)");
        gr.add_color_stop(0.5, "rgba(60,55,45,0.05)");
        gr.add_color_stop(1, "rgba(60,55,45,0.2)");
        ctx.set_fill_style(gr);
        ctx.fill_rect(0, 0, w, h);
    });
    lambert!({ map: &tex })
}
