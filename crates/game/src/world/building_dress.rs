//! The Building's floors dressed (`docs/MEGASTRUCTURE.md`, design 3): what
//! makes the concrete a place people lived in and left. All of it draws
//! from a salted stream of its own (`DRESS_SALT`), never from the build's,
//! so the floor's plan and every checkpoint stay where they are; none of it
//! stands in a body's way (a cell's front and a room's are solids of the
//! plan's, `level::building::cell_front_boxes`). The passes see a room off a
//! walk as the building it was cut from (`dress_ground`), so they draw as
//! they did before rooms, and leave out what would stand in a room's
//! doorway or over the top of the building it hangs on (`off_face`).
//!
//! * Every cell: its front wall and doorway, the hermetic door swung open
//!   inside, its wheel on either face, a plate with its number, the lamp in
//!   a cage over it, a bench.
//! * The Well: doors sealed from inside along every gallery, numbered, some
//!   marked red or taped round (what answers a knock: `world::knock`); the galleries of the storeys above and below
//!   yours, one over another up and down the face of the block, with their
//!   parapets and doors; laundry across the well; a lift cage hanging in
//!   the open void on its cables, its bulb still lit.
//! * Windows by the thousand on the panel faces, a few lit; the door's
//!   stair tower, its windows lit a storey apart up to the arrival's height.
//! * Storey numbers painted by the flights; loudspeakers on the walls.
//! * The Stair Halls: chains hanging out of the dark, flights of stairs that
//!   nobody can reach crossing the void, iron lanterns.
//! * The Works: a ceiling far up with its rows of dead fittings, gantries
//!   under it, pipes along the walls.
use crate::level::archgen::{build_ground_solids, segment_clear_3d, Ground, DIRS, GROUND_FLOOR, GROUND_MASS, GROUND_VOID};
use crate::level::building::{
    BuildingPlan, Style, CELL_DOOR, CELL_DOOR_H, CELL_FRONT, MASS_TOP, ROOM_DOOR, ROOM_DOOR_H, ROOM_HEADROOM, STOREY, VOID_DEPTH,
};
use crate::prelude::*;
use crate::visuals::textures::{text_tex, TextTexOpts};
use crate::world::knock::{KnockDoor, Mark};

/// The salt of the stream the dressing draws from
pub const DRESS_SALT: u64 = 0xB01D_0D2E_55ED_0001;
/// The rooms off the walks' own stream (`dress_rooms`): dressed last, from
/// a stream of their own, so the rest of the dressing draws as before
pub const ROOMS_SALT: u64 = 0xB01D_0D2E_55ED_0002;
/// How many storeys of galleries are drawn above and below a walk (the fog
/// takes the rest)
const STACK: i32 = 9;
/// The share of windows lit
const LIT_SHARE: f64 = 0.035;
/// Tiles of the floor to each chain hung in the Stair Halls and to each line
/// of washing across the Well (34 and 9 on the floors of 40 tiles a side
/// they were first hung on; the floors are 28 a side since G1, and as thick
/// with them as before)
const TILES_PER_CHAIN: i32 = 47;
const TILES_PER_LINE: i32 = 178;

/// The storey painted by a walk at height 0, place by place (the Building
/// counts its storeys in the thousands, and they count down as you go down)
pub(crate) fn storey_base(style: Style) -> i32 {
    match style {
        Style::Well => 1400,
        Style::Halls => 820,
        Style::Works => 260,
    }
}

/// Instanced parts gathered while dressing: (geometry, material, matrices)
#[derive(Default)]
struct Parts {
    list: Vec<(Geo, Mat, Vec<Mat4>)>,
}
impl Parts {
    /// Index of the part for `geo`/`mat` (created on first use)
    fn part(&mut self, geo: &Geo, mat: &Mat) -> usize {
        if let Some(i) = self.list.iter().position(|(g, m, _)| Geo::ptr_eq(g, geo) && Mat::ptr_eq(m, mat)) {
            return i;
        }
        self.list.push((geo.clone(), mat.clone(), vec![]));
        self.list.len() - 1
    }
    fn put(&mut self, k: usize, m: Mat4) {
        self.list[k].2.push(m);
    }
}

/// A box at `c` (centre), turned by `yaw`, of size `s`
fn boxm(c: Vec3, yaw: f64, s: Vec3) -> Mat4 {
    Mat4::compose(&c, &Quat::from_axis_angle(&Vec3::new(0.0, 1.0, 0.0), yaw), &s)
}

/// The yaw that turns an object's +z toward `DIRS[d]`
fn yaw_to(d: usize) -> f64 {
    (DIRS[d][0] as f64).atan2(DIRS[d][1] as f64)
}

/// A door wheel's ring and its spoke (shared by every door)
fn door_wheel_geos() -> (Geo, Geo) {
    thread_local! { static G: std::cell::RefCell<Option<(Geo, Geo)>> = const { std::cell::RefCell::new(None) }; }
    G.with(|g| g.borrow_mut().get_or_insert_with(|| (torus_geo!(0.2, 0.022, 6, 18), box_geo!(0.4, 0.028, 0.028))).clone())
}

impl Game {
    pub(crate) fn ground_or_mass(&self, x: i32, z: i32) -> Ground {
        if !self.in_grid(x, z) {
            return Ground::mass(VOID_DEPTH, MASS_TOP);
        }
        self.ground[self.t_idx(x, z)]
    }

    /// The ground the dressing sees at tile (x, z): the floor as built, but a
    /// room off a walk as the building it was cut from (`RoomPlan::was`), so
    /// every pass draws what it drew before rooms (`off_face` then leaves out
    /// what would land in a room)
    pub(crate) fn dress_ground(&self, plan: &BuildingPlan, x: i32, z: i32) -> Ground {
        match plan.rooms.iter().find(|r| r.tile == (x, z)) {
            Some(r) => r.was,
            None => self.ground_or_mass(x, z),
        }
    }

    /// Whether a thing from `y0` to `y1` on side `d` of the building at tile
    /// (x, z) (as `dress_ground` sees it) has no wall to stand on there: it is
    /// over that building's top (a booth stops a storey over its walk:
    /// `BOOTH_TOP`), in a room's doorway below its roof (`ROOM_HEADROOM` over
    /// its floor), or inside a room. The pass keeps its draws for it.
    fn off_face(&self, plan: &BuildingPlan, x: i32, z: i32, d: usize, y0: f64, y1: f64) -> bool {
        if y1 > self.dress_ground(plan, x, z).h1 as f64 {
            return true;
        }
        let (fx, fz) = (x + DIRS[d][0], z + DIRS[d][1]);
        plan.rooms.iter().any(|r| {
            let h = self.ground_or_mass(r.tile.0, r.tile.1).h0 as f64;
            let front = r.tile == (x, z) && d == r.door && y0 < h + ROOM_HEADROOM as f64 && y1 > h;
            front || (r.tile == (x, z) && d != r.door) || r.tile == (fx, fz)
        })
    }

    /// The point on the face of tile (x, z) on side `d`, `out` metres out
    /// from it, `along` metres to its right (looking at the face), at `y`
    fn face_point(&self, x: i32, z: i32, d: usize, along: f64, out: f64, y: f64) -> Vec3 {
        let (dx, dz) = (DIRS[d][0] as f64, DIRS[d][1] as f64);
        // right of the face, seen from outside it: the normal turned a quarter
        let (rx, rz) = (-dz, dx);
        Vec3::new(self.wx(x) + dx * (CELL / 2.0 + out) + rx * along, y, self.wz(z) + dz * (CELL / 2.0 + out) + rz * along)
    }

    /// Dress a floor of the Building (after its architecture and lamps)
    pub(crate) fn dress_building(&mut self, plan: &BuildingPlan) {
        let mut r = SplitMix64::new(self.current_seed, DRESS_SALT);
        let mut parts = Parts::default();
        self.dress_cells(plan, &mut parts, &mut r);
        if plan.style == Style::Well {
            self.dress_galleries(plan, &mut parts, &mut r);
            self.dress_laundry(plan, &mut parts, &mut r);
        }
        self.dress_windows(plan, &mut parts, &mut r);
        self.dress_numbers(plan, &mut r);
        self.dress_speakers(plan, &mut parts, &mut r);
        match plan.style {
            Style::Halls => self.dress_halls(plan, &mut parts, &mut r),
            Style::Works => self.dress_works(plan, &mut parts, &mut r),
            Style::Well => self.dress_lift(plan, &mut parts),
        }
        self.dress_rooms(plan, &mut parts);
        for (geo, mat, list) in parts.list {
            if !list.is_empty() {
                self.add_instanced(&geo, &mat, &list, &|m: &Mat4| *m);
            }
        }
    }

    /// A hermetic door: its frame in the opening (a box each side and one
    /// over it), its leaf (closed, or swung open `open` radians inward on
    /// its left hinge), the wheel on it. `at` is the middle of the opening's
    /// floor on the face, `yaw` turns +z out of the face.
    fn hermetic_door(&self, parts: &mut Parts, at: Vec3, yaw: f64, open: f64, steel: &Mat) {
        self.door_frame(parts, at, yaw, steel);
        let frame_geo = self.unit_box_geo.clone();
        let (c, s) = (yaw.cos(), yaw.sin());
        let to_w = |lx: f64, ly: f64, lz: f64| Vec3::new(at.x + lx * c + lz * s, at.y + ly, at.z - lx * s + lz * c);
        let (hw, hh) = (CELL_DOOR / 2.0, CELL_DOOR_H);
        // the leaf, hinged at its left edge
        let leaf = parts.part(&frame_geo, steel);
        let (lw, lh) = (CELL_DOOR - 0.06, hh - 0.04);
        // (turned by `open` about the hinge: inward, into the room)
        let hinge = (-hw + 0.03, -0.08);
        let a = open;
        let mid = (hinge.0 + (lw / 2.0) * a.cos(), hinge.1 - (lw / 2.0) * a.sin());
        parts.put(leaf, boxm(to_w(mid.0, lh / 2.0 + 0.02, mid.1), yaw + a, Vec3::new(lw, lh, 0.09)));
        // the wheel, on the leaf's outer face: a ring and two spokes
        let (ring_geo, spoke_geo) = door_wheel_geos();
        let ring = parts.part(&ring_geo, steel);
        let spoke = parts.part(&spoke_geo, steel);
        let wc = to_w(mid.0 + 0.07 * a.sin(), 1.05, mid.1 + 0.07 * a.cos());
        parts.put(ring, boxm(wc, yaw + a, Vec3::new(1.0, 1.0, 1.0)));
        for k in [0.0, PI / 2.0] {
            let mut q = Quat::from_axis_angle(&Vec3::new(0.0, 1.0, 0.0), yaw + a);
            q.multiply(&Quat::from_axis_angle(&Vec3::new(0.0, 0.0, 1.0), k + 0.4));
            parts.put(spoke, Mat4::compose(&wc, &q, &Vec3::new(1.0, 1.0, 1.0)));
        }
    }

    /// A doorway's steel frame: a box each side and one over it
    fn door_frame(&self, parts: &mut Parts, at: Vec3, yaw: f64, steel: &Mat) {
        let frame_geo = self.unit_box_geo.clone();
        let frame = parts.part(&frame_geo, steel);
        let (c, s) = (yaw.cos(), yaw.sin());
        // local (x along the face to the right seen from outside, z out) → world
        let to_w = |lx: f64, ly: f64, lz: f64| Vec3::new(at.x + lx * c + lz * s, at.y + ly, at.z - lx * s + lz * c);
        let (hw, hh) = (CELL_DOOR / 2.0, CELL_DOOR_H);
        for sx in [-1.0, 1.0] {
            parts.put(frame, boxm(to_w(sx * (hw + 0.06), hh / 2.0, 0.02), yaw, Vec3::new(0.12, hh, 0.32)));
        }
        parts.put(frame, boxm(to_w(0.0, hh + 0.06, 0.02), yaw, Vec3::new(CELL_DOOR + 0.24, 0.12, 0.32)));
    }

    /// A cell's door leaf and its wheel, as an object that turns on its
    /// hinge (its rotation is the face's `yaw` plus how far it stands open),
    /// and the wheel on its spindle through the leaf, a ring and two spokes
    /// on either face (it turns about its local z: `world::cell_knock`)
    fn door_leaf(&self, at: Vec3, yaw: f64, steel: &Mat, worn: &Mat) -> (Obj, Obj) {
        let (c, s) = (yaw.cos(), yaw.sin());
        let (hw, hh) = (CELL_DOOR / 2.0, CELL_DOOR_H);
        let (lw, lh) = (CELL_DOOR - 0.06, hh - 0.04);
        let (hx, hz) = (-hw + 0.03, -0.08);
        let g = group();
        g.position().set(at.x + hx * c + hz * s, at.y, at.z - hx * s + hz * c);
        g.rotation().y = yaw;
        let leaf = mesh(&self.unit_box_geo, steel);
        leaf.position().set(lw / 2.0, lh / 2.0 + 0.02, 0.0);
        leaf.scale().set(lw, lh, 0.09);
        g.add(&leaf);
        let (ring_geo, spoke_geo) = door_wheel_geos();
        let wheel = group();
        wheel.position().set(lw / 2.0, 1.05, 0.0);
        // outside, and inside (the one turned from in the cell)
        for z in [0.07, -0.07] {
            let ring = mesh(&ring_geo, worn);
            ring.position().z = z;
            wheel.add(&ring);
            for k in [0.0, PI / 2.0] {
                let sp = mesh(&spoke_geo, worn);
                sp.position().z = z;
                sp.rotation().z = k + 0.4;
                wheel.add(&sp);
            }
        }
        g.add(&wheel);
        g.user().gameplay = true;
        g.user().set("yaw", yaw);
        (g, wheel)
    }

    /// The places off the walks (G4, `level::building::place_rooms`): a
    /// room cut into the building beside a walk, closed over and under as a
    /// cell is, its front a wide doorway with no door; furnished as its
    /// place furnishes (a resident's table and chair in the Well, a stone
    /// bench in the Stair Halls' piers, an operator's desk, its dead screen
    /// and a stool in the Works), and on it water or a battery
    fn dress_rooms(&mut self, plan: &BuildingPlan, parts: &mut Parts) {
        if plan.rooms.is_empty() {
            return;
        }
        let mut r = SplitMix64::new(self.current_seed, ROOMS_SALT);
        let wall_mat = self.arch_mat(&format!("{}:1", self.building_wall_kind()));
        let unit = self.unit_box_geo.clone();
        let wall = parts.part(&unit, &wall_mat);
        let wood = parts.part(&unit, &lambert!({ color: 0x5a4632 }));
        // (darker than the piers it is set in, or it is lost against them)
        let stone = parts.part(&unit, &lambert!({ color: 0x3b3731 }));
        let steel = parts.part(&unit, &lambert!({ color: 0x4f5550 }));
        let screen = parts.part(&unit, &lambert!({ color: 0x1d2220 }));
        let bottle_geo = pg("Props__almond_bottle", cyl_geo!(0.09, 0.09, 0.3, 10));
        let bottle_mat = lambert!({ color: 0x9fc4d8, emissive: 0x16323e });
        let cap_geo = pg("Props__almond_cap", cyl_geo!(0.05, 0.05, 0.06, 8));
        let bat_body_geo = pg("Props__battery_body", cyl_geo!(0.06, 0.06, 0.18, 10));
        let bat_tip_geo = pg("Props__battery_tip", cyl_geo!(0.025, 0.025, 0.03, 8));
        let bat_body_mat = lambert!({ color: 0xc79b3d, emissive: 0x2a1f08 });
        let bat_tip_mat = lambert!({ color: 0xdad3c2 });
        for (i, c) in plan.rooms.iter().enumerate() {
            let (x, z) = c.tile;
            let h = self.floor_y_of(x, z);
            let top = DIRS
                .iter()
                .map(|dd| self.ground_or_mass(x + dd[0], z + dd[1]))
                .filter(|g| g.kind == GROUND_MASS)
                .map(|g| g.h1 as f64)
                .fold(h + 3.0, f64::max);
            // the building over and under the room
            let (cx, cz) = (self.wx(x), self.wz(z));
            parts.put(wall, boxm(Vec3::new(cx, (h + 3.0 + top) / 2.0, cz), 0.0, Vec3::new(CELL, top - h - 3.0, CELL)));
            parts.put(
                wall,
                boxm(Vec3::new(cx, (VOID_DEPTH as f64 + h - 0.4) / 2.0, cz), 0.0, Vec3::new(CELL, h - 0.4 - VOID_DEPTH as f64, CELL)),
            );
            // the front: a wide doorway, the wall either side and over it
            let d = c.door;
            let yaw = yaw_to(d);
            // (`level::building::cell_front_boxes` makes it solid)
            let seg = (CELL - ROOM_DOOR) / 2.0;
            for sx in [-1.0, 1.0] {
                let p = self.face_point(x, z, d, sx * (ROOM_DOOR / 2.0 + seg / 2.0), -CELL_FRONT / 2.0, (h + top) / 2.0);
                parts.put(wall, boxm(p, yaw, Vec3::new(seg, top - h, CELL_FRONT)));
            }
            let p = self.face_point(x, z, d, 0.0, -CELL_FRONT / 2.0, (h + ROOM_DOOR_H + top) / 2.0);
            parts.put(wall, boxm(p, yaw, Vec3::new(ROOM_DOOR, top - h - ROOM_DOOR_H, CELL_FRONT)));
            // what its place leaves in it, at the back, a little to one side
            let side = if r.next() < 0.5 { -1.0 } else { 1.0 };
            let back = |along: f64, out: f64, y: f64| self.face_point(x, z, d ^ 1, along, out, y);
            let on = match plan.style {
                Style::Well => {
                    // a resident's table, and the chair pushed back from it
                    let t = back(side * 0.6, -0.75, h + 0.73);
                    parts.put(wood, boxm(t, yaw, Vec3::new(1.1, 0.05, 0.7)));
                    for (lx, lz) in [(-0.5, -0.3), (0.5, -0.3), (-0.5, 0.3), (0.5, 0.3)] {
                        let leg = back(side * 0.6 + lx, -0.75 + lz, h + 0.36);
                        parts.put(wood, boxm(leg, yaw, Vec3::new(0.05, 0.72, 0.05)));
                    }
                    // (seat and back turned together about the seat's middle)
                    let seat = back(side * 0.6 - side * 0.2, -1.55, h + 0.45);
                    let turn = r.range(-0.4, 0.4);
                    parts.put(wood, boxm(seat, yaw + turn, Vec3::new(0.45, 0.05, 0.45)));
                    let rest = back(side * 0.6 - side * 0.2, -1.76, h + 0.75);
                    let (ox, oz) = (rest.x - seat.x, rest.z - seat.z);
                    let (c, s) = (turn.cos(), turn.sin());
                    let rest = Vec3::new(seat.x + ox * c + oz * s, rest.y, seat.z - ox * s + oz * c);
                    parts.put(wood, boxm(rest, yaw + turn, Vec3::new(0.45, 0.6, 0.05)));
                    t + Vec3::new(0.0, 0.025, 0.0)
                }
                Style::Halls => {
                    // a stone bench along the back of a niche in the pier
                    let b = back(0.0, -0.35, h + 0.22);
                    parts.put(stone, boxm(b, yaw, Vec3::new(2.4, 0.44, 0.6)));
                    back(side * 0.7, -0.35, h + 0.44)
                }
                Style::Works => {
                    // the operator's desk, its dead screen, a stool
                    let t = back(side * 0.5, -0.5, h + 0.76);
                    parts.put(steel, boxm(t - Vec3::new(0.0, 0.38, 0.0), yaw, Vec3::new(1.4, 0.76, 0.7)));
                    let sc = back(side * 0.5 + 0.35, -0.35, h + 0.97);
                    parts.put(screen, boxm(sc, yaw, Vec3::new(0.42, 0.36, 0.34)));
                    let stool = back(side * 0.5 - 0.1, -1.4, h + 0.3);
                    parts.put(steel, boxm(stool, 0.0, Vec3::new(0.36, 0.6, 0.36)));
                    back(side * 0.5 - 0.3, -0.5, h + 0.76)
                }
            };
            // and on it, water or a battery in turn
            let water = i % 2 == 0;
            let base = on.y + if water { 0.15 } else { 0.11 };
            let gp = group();
            gp.position().set(on.x, base, on.z);
            if water {
                gp.add(&mesh(&bottle_geo, &bottle_mat));
                let cap = mesh(&cap_geo, &self.mats.metal);
                cap.position().y = 0.18;
                gp.add(&cap);
            } else {
                gp.add(&mesh(&bat_body_geo, &bat_body_mat));
                let tip = mesh(&bat_tip_geo, &bat_tip_mat);
                tip.position().y = 0.105;
                gp.add(&tip);
            }
            self.world.add(&gp);
            let p = crate::base::records::Pickup { g: gp, taken: false, base, phase: r.range(0.0, 7.0) };
            if water {
                self.almonds.push(p);
            } else {
                self.battery_items.push(p);
            }
        }
    }

    /// Every cell: the front wall with its doorway, the door swung open
    /// inside, its number, the lamp's cage, a bench at the back; the room
    /// closed over and under (the building it is cut into)
    fn dress_cells(&mut self, plan: &BuildingPlan, parts: &mut Parts, r: &mut SplitMix64) {
        let wall_mat = self.arch_mat(&format!("{}:1", self.building_wall_kind()));
        let unit = self.unit_box_geo.clone();
        let wall = parts.part(&unit, &wall_mat);
        let steel = lambert!({ color: 0x4d5650 });
        // (a cell's wheel is bare where hands have turned it: paler than its
        // painted leaf, so it reads in the dark of the cell)
        let worn = lambert!({ color: 0x80857e });
        let dark = lambert!({ color: 0x2c2f2c });
        let bench = parts.part(&unit, &dark);
        let cage = parts.part(&unit, &self.mats.metal);
        for (i, c) in plan.cells.iter().enumerate() {
            let (x, z) = c.tile;
            let h = self.floor_y_of(x, z);
            let top = DIRS
                .iter()
                .map(|dd| self.ground_or_mass(x + dd[0], z + dd[1]))
                .filter(|g| g.kind == GROUND_MASS)
                .map(|g| g.h1 as f64)
                .fold(h + 3.0, f64::max);
            // the building over and under the room
            let (cx, cz) = (self.wx(x), self.wz(z));
            parts.put(wall, boxm(Vec3::new(cx, (h + 3.0 + top) / 2.0, cz), 0.0, Vec3::new(CELL, top - h - 3.0, CELL)));
            parts.put(
                wall,
                boxm(Vec3::new(cx, (VOID_DEPTH as f64 + h - 0.4) / 2.0, cz), 0.0, Vec3::new(CELL, h - 0.4 - VOID_DEPTH as f64, CELL)),
            );
            // the front, either side of the doorway and over it
            let d = c.door;
            let yaw = yaw_to(d);
            let seg = (CELL - CELL_DOOR) / 2.0;
            for sx in [-1.0, 1.0] {
                let p = self.face_point(x, z, d, sx * (CELL_DOOR / 2.0 + seg / 2.0), -CELL_FRONT / 2.0, (h + top) / 2.0);
                parts.put(wall, boxm(p, yaw, Vec3::new(seg, top - h, CELL_FRONT)));
            }
            let p = self.face_point(x, z, d, 0.0, -CELL_FRONT / 2.0, (h + CELL_DOOR_H + top) / 2.0);
            parts.put(wall, boxm(p, yaw, Vec3::new(CELL_DOOR, top - h - CELL_DOOR_H, CELL_FRONT)));
            // the door, open into the room; turn its wheel from inside to
            // shut it (the shelter from the self-assembly), and again to open
            let at = self.face_point(x, z, d, 0.0, -CELL_FRONT / 2.0, h);
            self.door_frame(parts, at, yaw, &steel);
            let (leaf, wheel) = self.door_leaf(at, yaw, &steel, &worn);
            let open = crate::world::assembly::OPEN + r.range(-0.1, 0.1);
            leaf.rotation().y = yaw + open;
            self.world.add(&leaf);
            let (ax, az) = if DIRS[d][0] != 0 { (CELL_FRONT / 2.0, CELL_DOOR / 2.0) } else { (CELL_DOOR / 2.0, CELL_FRONT / 2.0) };
            let ci = self.cell_doors.len();
            self.cell_doors.push(crate::world::assembly::CellDoor {
                tile: (x, z),
                leaf,
                open,
                target: open,
                at,
                span: [at.x - ax, at.z - az, at.x + ax, at.z + az],
                wheel,
                turned: 0.0,
                turn_to: 0.0,
                strain: 0.0,
                shake: 0.0,
            });
            let inside = self.face_point(x, z, d, 0.0, -1.1, h + EYE);
            let in_cell = move |g: &Game| (g.tx(g.player.pos.x), g.tz(g.player.pos.z)) == (x, z);
            // (not while something is at the door, and a door that gave not
            // until the fog is gone: `world::cell_knock`)
            self.interact(
                inside,
                1.7,
                "[E] TURN THE WHEEL",
                move |g| in_cell(g) && g.cell_doors.get(ci).is_some_and(|d| d.target > 0.5) && !g.cell_door_gave(ci),
                move |g| g.turn_cell_wheel(ci, true),
            );
            self.interact(
                inside,
                1.7,
                "[E] OPEN THE DOOR",
                move |g| in_cell(g) && g.cell_doors.get(ci).is_some_and(|d| d.target < 0.5) && !g.at_the_door(ci),
                move |g| g.turn_cell_wheel(ci, false),
            );
            // what knocks at the door while the fog is up: knock back at the
            // door (the knock of `world::knock`), and hold the wheel when it
            // turns (`world::cell_knock`)
            let knock_at = self.face_point(x, z, d, 0.0, -0.75, h + EYE);
            self.interact(knock_at, 1.5, "[E] KNOCK", move |g| in_cell(g) && g.cell_knocking(ci), move |g| g.answer_the_door(ci));
            self.interact(inside, 1.7, "[E] HOLD THE WHEEL", move |g| in_cell(g) && g.cell_wheel_tried(ci), move |g| g.hold_the_wheel(ci));
            // the lamp's cage over the door, outside
            let lamp = self.face_point(x, z, d, 0.0, 0.15, h + crate::world::building::CELL_LAMP_Y);
            for (sx, sy, sz) in [(0.26, 0.02, 0.02), (0.02, 0.2, 0.02)] {
                parts.put(cage, boxm(lamp, yaw, Vec3::new(sx, sy, sz)));
            }
            // a bench along the back wall
            let back = self.face_point(x, z, d ^ 1, 0.0, -0.3, h + 0.22);
            parts.put(bench, boxm(back, yaw, Vec3::new(2.6, 0.44, 0.5)));
            // its number, by the door
            let n = 100 + (i as u32 * 37 + (r.next() * 30.0) as u32) % 900;
            let letter = ['A', 'B', 'C', 'D'][(r.next() * 4.0) as usize % 4];
            let name = format!("{n}-{letter}");
            let plate = self.plate(&name, "#d8d2c0", "#30352f");
            // (what a resident's note names it by: `world::knock`)
            self.knocks.cell_plates.push(name);
            let pp = self.face_point(x, z, d, CELL_DOOR / 2.0 + 0.45, 0.012, h + 1.7);
            plate.position().copy(&pp);
            plate.rotation().y = yaw;
            self.world.add(&plate);
        }
    }

    /// The wall kind of the floor's cells: the place's own, but poured
    /// concrete in the Well (its panels are faces with windows)
    fn building_wall_kind(&self) -> &'static str {
        match self.layout.as_ref().and_then(|l| l.zones.first()).map(|z| z.wall_mat) {
            Some("panel") | None => "poured",
            Some(k) => k,
        }
    }

    /// A small painted plate with `text` on it
    fn plate(&self, text: &str, ink: &str, ground: &str) -> Obj {
        let tex = text_tex(
            text,
            &TextTexOpts {
                w: 128.0,
                h: 64.0,
                font: "bold 40px Helvetica, Arial, sans-serif".into(),
                color: ink.into(),
                glow: 0.0,
                rot: 0.0,
            },
        );
        let g = group();
        let back = mesh(plane_geo!(0.34, 0.17), lambert!({ color: ground }));
        g.add(&back);
        let face = mesh(plane_geo!(0.32, 0.16), lambert!({ map: tex, transparent: true }));
        face.position().z = 0.004;
        g.add(&face);
        g.user().static_prop = true;
        g
    }

    /// The Well's galleries: doors sealed from inside along the faces of the
    /// block by every walk, and the galleries of the storeys above and below
    /// it, one over another, up and down the face
    fn dress_galleries(&mut self, plan: &BuildingPlan, parts: &mut Parts, r: &mut SplitMix64) {
        let unit = self.unit_box_geo.clone();
        let deck_mat = self.arch_mat("floor:deck:2");
        let wall_mat = self.arch_mat("poured:2");
        let deck = parts.part(&unit, &deck_mat);
        let parapet = parts.part(&unit, &wall_mat);
        let steel = lambert!({ color: 0x47504a });
        let red = lambert!({ color: 0x7a2a22 });
        let tape = lambert!({ color: 0x9a927c });
        let marks = parts.part(&unit, &red);
        let taped = parts.part(&unit, &tape);
        let cells: Vec<(i32, i32)> = plan.cells.iter().map(|c| c.tile).collect();
        let exit_wall = (plan.exit.0 + DIRS[plan.exit_side][0], plan.exit.1 + DIRS[plan.exit_side][1]);
        // (a door's height with its frame)
        let door_h = CELL_DOOR_H + 0.12;
        for z in 0..self.h {
            for x in 0..self.w {
                let g = self.dress_ground(plan, x, z);
                if g.kind != GROUND_FLOOR || cells.contains(&(x, z)) || plan.view_line.contains(&(x, z)) {
                    continue;
                }
                let h = g.h0 as f64;
                let faces: Vec<usize> = (0..4)
                    .filter(|&d| {
                        let (mx, mz) = (x + DIRS[d][0], z + DIRS[d][1]);
                        self.dress_ground(plan, mx, mz).kind == GROUND_MASS
                            && !cells.contains(&(mx, mz))
                            && (mx, mz) != exit_wall
                            && self.in_grid(mx, mz)
                    })
                    .collect();
                let voids: Vec<usize> =
                    (0..4).filter(|&d| self.dress_ground(plan, x + DIRS[d][0], z + DIRS[d][1]).kind == GROUND_VOID).collect();
                if faces.is_empty() {
                    continue;
                }
                // a door on each face of the block along the walk, sealed (none
                // in a room's doorway: its draws are kept)
                for &d in &faces {
                    if r.next() > 0.6 {
                        continue;
                    }
                    let (mx, mz) = (x + DIRS[d][0], z + DIRS[d][1]);
                    let yaw = yaw_to(d ^ 1);
                    // (its mark and number drawn even where it is left out)
                    let v = r.next();
                    let n = 1000 + ((x * 31 + z * 17) as u32 * 7 + (r.next() * 9.0) as u32) % 8000;
                    if self.off_face(plan, mx, mz, d ^ 1, h, h + door_h) {
                        continue;
                    }
                    let at = self.face_point(mx, mz, d ^ 1, 0.0, 0.04, h);
                    self.hermetic_door(parts, at, yaw, 0.0, &steel);
                    let mark = match v {
                        // the cleaners' mark: painted red across it
                        v if v < 0.12 => {
                            for k in [-1.0, 1.0] {
                                let c = self.face_point(mx, mz, d ^ 1, 0.0, 0.11, h + 1.05);
                                let q = {
                                    let mut q = Quat::from_axis_angle(&Vec3::new(0.0, 1.0, 0.0), yaw);
                                    q.multiply(&Quat::from_axis_angle(&Vec3::new(0.0, 0.0, 1.0), k * 0.95));
                                    q
                                };
                                parts.put(marks, Mat4::compose(&c, &q, &Vec3::new(1.5, 0.09, 0.01)));
                            }
                            Mark::Red
                        }
                        // taped round, from inside out
                        v if v < 0.22 => {
                            for (lx, ly, sx, sy) in [
                                (0.0, CELL_DOOR_H, CELL_DOOR + 0.1, 0.07),
                                (-(CELL_DOOR / 2.0), CELL_DOOR_H / 2.0, 0.07, CELL_DOOR_H),
                                (CELL_DOOR / 2.0, CELL_DOOR_H / 2.0, 0.07, CELL_DOOR_H),
                            ] {
                                let c = self.face_point(mx, mz, d ^ 1, lx, 0.1, h + ly);
                                parts.put(taped, boxm(c, yaw, Vec3::new(sx, sy, 0.005)));
                            }
                            Mark::Taped
                        }
                        _ => Mark::Plain,
                    };
                    let plate = self.plate(&format!("{n}"), "#cfc9b6", "#2e332e");
                    plate.position().copy(&self.face_point(mx, mz, d ^ 1, CELL_DOOR / 2.0 + 0.4, 0.012, h + 1.65));
                    plate.rotation().y = yaw;
                    self.world.add(&plate);
                    // a knock at it (`world::knock`)
                    let normal = (DIRS[d ^ 1][0] as f64, DIRS[d ^ 1][1] as f64);
                    self.knocks.doors.push(KnockDoor {
                        tile: (x, z),
                        side: d,
                        number: n,
                        mark,
                        foot: self.face_point(mx, mz, d ^ 1, 0.0, 0.0, h),
                        normal,
                        right: (-normal.1, normal.0),
                        gift: None,
                        words: String::new(),
                        lie: (0.0, 0.0, 0.0),
                        answered: false,
                    });
                }
                // the galleries above and below, with their parapets and doors:
                // a gallery only as high as a face it hangs on (a booth stops a
                // storey over its walk), a door only on a face that holds it
                let reach =
                    faces.iter().map(|&d| self.dress_ground(plan, x + DIRS[d][0], z + DIRS[d][1]).h1 as f64).fold(f64::MIN, f64::max);
                for k in (-STACK..=STACK).filter(|&k| k != 0) {
                    let y = h + k as f64 * STOREY;
                    if y < VOID_DEPTH as f64 + 2.0 || y > MASS_TOP as f64 - 3.0 {
                        continue;
                    }
                    let hung = y <= reach;
                    let (cx, cz) = (self.wx(x), self.wz(z));
                    if hung {
                        parts.put(deck, boxm(Vec3::new(cx, y - 0.15, cz), 0.0, Vec3::new(CELL, 0.3, CELL)));
                        // where the figure across the void may stand: at the deck's
                        // parapet, a storey under to four over (`world::far_figures`)
                        if let (Some(&d), true) = (voids.first(), (-1..=4).contains(&k)) {
                            let at = Vec3::new(cx + DIRS[d][0] as f64 * 1.5, y, cz + DIRS[d][1] as f64 * 1.5);
                            self.figure_perches.push(at);
                        }
                        for &d in &voids {
                            let p = self.face_point(x, z, d, 0.0, -0.09, y + 0.52);
                            parts.put(parapet, boxm(p, yaw_to(d), Vec3::new(CELL, 1.05, 0.18)));
                        }
                    }
                    for &d in &faces {
                        let (mx, mz) = (x + DIRS[d][0], z + DIRS[d][1]);
                        if r.next() < 0.6 && hung && !self.off_face(plan, mx, mz, d ^ 1, y, y + door_h) {
                            let at = self.face_point(mx, mz, d ^ 1, 0.0, 0.04, y);
                            self.hermetic_door(parts, at, yaw_to(d ^ 1), 0.0, &steel);
                        }
                    }
                }
            }
        }
        // a knock on a sealed door: who answers, and how, is `world::knock`'s
        for i in 0..self.knocks.doors.len() {
            let at = self.knocks.doors[i].stand();
            self.interact(at, 1.3, "[E] KNOCK", |_g| true, move |g| g.knock_at(i));
        }
    }

    /// A lift cage hanging in the open well on its cables, stopped between
    /// storeys, its bulb still lit (a landmark over the void: design 7).
    /// Where the void is three tiles wide every way, the one nearest 45 m
    /// from the arrival, in sight of a walk near it if one is; none on a floor
    /// without such a void. It draws nothing.
    fn dress_lift(&mut self, plan: &BuildingPlan, parts: &mut Parts) {
        let (sx, sz) = (self.wx(plan.spawn.0), self.wz(plan.spawn.1));
        let open = |g: &Game, x: i32, z: i32| {
            (-1..=1).all(|dz| (-1..=1).all(|dx| g.in_grid(x + dx, z + dz) && g.dress_ground(plan, x + dx, z + dz).kind == GROUND_VOID))
        };
        // the eyes of the walks round the arrival
        let eyes: Vec<Vec3> = (-4..=4)
            .flat_map(|dz| (-4..=4).map(move |dx| (plan.spawn.0 + dx, plan.spawn.1 + dz)))
            .filter(|&(x, z)| self.in_grid(x, z) && self.dress_ground(plan, x, z).walkable())
            .map(|(x, z)| Vec3::new(self.wx(x), self.floor_y_of(x, z) + EYE, self.wz(z)))
            .collect();
        let eye_y = self.floor_y_of(plan.spawn.0, plan.spawn.1) + EYE;
        let solids = build_ground_solids(self.w, self.h, CELL, &self.ground);
        let mut best: Option<(f64, Vec3)> = None;
        for z in 1..self.h - 1 {
            for x in 1..self.w - 1 {
                if !open(self, x, z) || plan.view_line.contains(&(x, z)) {
                    continue;
                }
                let (cx, cz) = (self.wx(x), self.wz(z));
                let d = hypot2(cx - sx, cz - sz);
                if !(20.0..80.0).contains(&d) {
                    continue;
                }
                // stopped with its floor 1.5 m over a walk near the arrival
                // that sees it (the decks over the galleries hide much)
                let seen = eyes.iter().find_map(|e| {
                    let y = (e.y - EYE) + 1.5;
                    let mid = Vec3::new(cx, y + 1.3, cz);
                    (segment_clear_3d(&solids, self.to_u(e.x), self.to_v(e.z), e.y, self.to_u(cx), self.to_v(cz), mid.y)
                        && self.decks_clear(*e, mid))
                    .then_some(y)
                });
                let y = seen.unwrap_or((eye_y - EYE) + 1.5);
                let score = (d - 45.0).abs() + if seen.is_some() { 0.0 } else { 100.0 };
                if best.is_none_or(|b| score < b.0) {
                    best = Some((score, Vec3::new(cx, y, cz)));
                }
            }
        }
        let Some((_, at)) = best else { return };
        self.lift_at = Some(at);
        let unit = self.unit_box_geo.clone();
        let steel = lambert!({ color: 0x3e443f });
        let frame = parts.part(&unit, &steel);
        let (w, h) = (2.6, 2.7);
        let hw = w / 2.0;
        // floor, roof, corner posts
        parts.put(frame, boxm(Vec3::new(at.x, at.y, at.z), 0.0, Vec3::new(w + 0.1, 0.16, w + 0.1)));
        parts.put(frame, boxm(Vec3::new(at.x, at.y + h, at.z), 0.0, Vec3::new(w + 0.1, 0.12, w + 0.1)));
        for (ox, oz) in [(-hw, -hw), (hw, -hw), (-hw, hw), (hw, hw)] {
            parts.put(frame, boxm(Vec3::new(at.x + ox, at.y + h / 2.0, at.z + oz), 0.0, Vec3::new(0.09, h, 0.09)));
        }
        // the bars: three sides, and a gate drawn half across the fourth
        let n = 9;
        for k in 1..n {
            let t = -hw + w * k as f64 / n as f64;
            for (bx, bz, along_x) in [(t, -hw, true), (t, hw, true), (-hw, t, false)] {
                let _ = along_x;
                parts.put(frame, boxm(Vec3::new(at.x + bx, at.y + h / 2.0, at.z + bz), 0.0, Vec3::new(0.04, h, 0.04)));
            }
            if k <= n / 2 {
                parts.put(frame, boxm(Vec3::new(at.x + hw, at.y + h / 2.0, at.z + t), 0.0, Vec3::new(0.04, h, 0.04)));
            }
        }
        for y in [0.9, 1.8] {
            for (ox, oz, sx2, sz2) in [(0.0, -hw, w, 0.05), (0.0, hw, w, 0.05), (-hw, 0.0, 0.05, w)] {
                parts.put(frame, boxm(Vec3::new(at.x + ox, at.y + y, at.z + oz), 0.0, Vec3::new(sx2, 0.05, sz2)));
            }
        }
        // its cables up out of sight, and the counterweight's
        let cable_geo = cyl_geo!(0.03, 0.03, 1.0, 4);
        let cable = parts.part(&cable_geo, &self.mats.metal);
        let top = MASS_TOP as f64;
        for (ox, oz) in [(-0.4, 0.0), (0.4, 0.0), (0.0, hw + 0.7)] {
            let y0 = at.y + h + if oz > 1.0 { 9.0 } else { 0.0 };
            parts.put(
                cable,
                Mat4::compose(&Vec3::new(at.x + ox, (y0 + top) / 2.0, at.z + oz), &Quat::IDENTITY, &Vec3::new(1.0, top - y0, 1.0)),
            );
        }
        parts.put(frame, boxm(Vec3::new(at.x, at.y + h + 8.2, at.z + hw + 0.7), 0.0, Vec3::new(0.7, 1.6, 0.3)));
        // its bulb, still on
        let bulb_mat = basic_mat!({ color: 0xffe2a8 });
        let bulb = parts.part(&unit, &bulb_mat);
        parts.put(bulb, boxm(Vec3::new(at.x, at.y + h - 0.12, at.z), 0.0, Vec3::new(0.34, 0.1, 0.34)));
    }

    /// Washing on lines across the well, between two faces of the block
    fn dress_laundry(&mut self, plan: &BuildingPlan, parts: &mut Parts, r: &mut SplitMix64) {
        let line_geo = cyl_geo!(0.008, 0.008, 1.0, 4);
        let line = parts.part(&line_geo, &self.mats.metal);
        let cloths = [
            lambert!({ color: 0x8a8678, side: DoubleSide }),
            lambert!({ color: 0x6d7470, side: DoubleSide }),
            lambert!({ color: 0x7c6a5a, side: DoubleSide }),
        ];
        let cloth_geo = plane_geo!(0.7, 0.9);
        let cloth: Vec<usize> = cloths.iter().map(|m| parts.part(&cloth_geo, m)).collect();
        let mut made = 0;
        for _ in 0..400 {
            if made >= self.w * self.h / TILES_PER_LINE {
                break;
            }
            let x = (r.next() * self.w as f64) as i32;
            let z = (r.next() * self.h as f64) as i32;
            let along_x = r.next() < 0.5;
            let (dx, dz) = if along_x { (1, 0) } else { (0, 1) };
            if self.dress_ground(plan, x, z).kind != GROUND_MASS || self.dress_ground(plan, x + dx, z + dz).kind != GROUND_VOID {
                continue;
            }
            // across the void to the next face
            let mut k = 1;
            while k < 12
                && self.dress_ground(plan, x + dx * k, z + dz * k).kind == GROUND_VOID
                && !plan.view_line.contains(&(x + dx * k, z + dz * k))
            {
                k += 1;
            }
            if !(3..12).contains(&k) || self.dress_ground(plan, x + dx * k, z + dz * k).kind != GROUND_MASS {
                continue;
            }
            let y = ((r.next() * 10.0 - 4.0).round()) * STOREY + 2.3;
            // tied to a face at either end: where one stops under it (a
            // booth), the line is left out and its draws kept
            let d = if along_x { 0 } else { 2 };
            let tied = !self.off_face(plan, x, z, d, y, y) && !self.off_face(plan, x + dx * k, z + dz * k, d ^ 1, y, y);
            let a = Vec3::new(self.wx(x) + dx as f64 * CELL / 2.0, y, self.wz(z) + dz as f64 * CELL / 2.0);
            let b = Vec3::new(self.wx(x + dx * k) - dx as f64 * CELL / 2.0, y, self.wz(z + dz * k) - dz as f64 * CELL / 2.0);
            let len = a.distance_to(&b);
            let mid = Vec3::new((a.x + b.x) / 2.0, y - 0.15, (a.z + b.z) / 2.0);
            let q = Quat::from_axis_angle(&Vec3::new(dz as f64, 0.0, -dx as f64), PI / 2.0);
            if tied {
                parts.put(line, Mat4::compose(&mid, &q, &Vec3::new(1.0, len, 1.0)));
            }
            let n = 2 + (r.next() * 5.0) as usize;
            for j in 0..n {
                let t = (j as f64 + 0.5 + r.range(-0.3, 0.3)) / n as f64;
                let p = Vec3::new(a.x + (b.x - a.x) * t, y - 0.6 - r.range(0.0, 0.1), a.z + (b.z - a.z) * t);
                let yaw = if along_x { 0.0 } else { PI / 2.0 };
                let m = cloth[(r.next() * cloth.len() as f64) as usize % cloth.len()];
                let turn = r.range(-0.1, 0.1);
                if tied {
                    parts.put(m, boxm(p, yaw + turn, Vec3::new(1.0, 1.0, 1.0)));
                }
            }
            // (counted made either way: the draws after it stay where they were)
            made += 1;
        }
    }

    /// Lit windows on the panel faces (the Well), and the door's tower
    fn dress_windows(&mut self, plan: &BuildingPlan, parts: &mut Parts, r: &mut SplitMix64) {
        let geo = plane_geo!(1.42, 1.32);
        // a photographed window lit from inside (`visuals::photo`), in a lamp's
        // warm light, a cold one, or a television's blue: tint by tint, window by window
        use crate::visuals::photo::{lit_window_tex, LIT_CELLS};
        let mut lit = [[0usize; LIT_CELLS]; 3];
        for (k, tint) in [0xd9b67a, 0xc9bfa2, 0x8e9bb0].into_iter().enumerate() {
            for (cell, slot) in lit[k].iter_mut().enumerate() {
                *slot = parts.part(&geo, &basic_mat!({ color: tint, map: lit_window_tex(cell) }));
            }
        }
        // which of a tint's windows: the draw's place inside the tint's share
        let pick = |v: f64, lo: f64, hi: f64| (((v - lo) / (hi - lo)) * LIT_CELLS as f64).clamp(0.0, LIT_CELLS as f64 - 1.0) as usize;
        if plan.style == Style::Well {
            for z in 0..self.h {
                for x in 0..self.w {
                    if self.dress_ground(plan, x, z).kind != GROUND_MASS {
                        continue;
                    }
                    for d in 0..4 {
                        let (nx, nz) = (x + DIRS[d][0], z + DIRS[d][1]);
                        if !self.in_grid(nx, nz) || self.dress_ground(plan, nx, nz).kind == GROUND_MASS {
                            continue;
                        }
                        let out = self.dress_ground(plan, nx, nz);
                        for k in ((VOID_DEPTH as f64 / STOREY) as i32 + 2)..((MASS_TOP as f64 / STOREY) as i32 - 2) {
                            let y = k as f64 * STOREY + STOREY / 2.0;
                            // not beside a walk, at its height: those are doors
                            if out.walkable() && (y - out.h0 as f64).abs() < 2.0 {
                                continue;
                            }
                            if r.next() > LIT_SHARE {
                                continue;
                            }
                            let p = self.face_point(x, z, d, 0.0, 0.015, y);
                            let m = match r.next() {
                                v if v < 0.7 => lit[0][pick(v, 0.0, 0.7)],
                                v if v < 0.95 => lit[1][pick(v, 0.7, 0.95)],
                                v => lit[2][pick(v, 0.95, 1.0)],
                            };
                            // (none over the top of a booth, nor in a room's
                            // doorway: the draws are kept)
                            if self.off_face(plan, x, z, d, y - 0.66, y + 0.66) {
                                continue;
                            }
                            parts.put(m, boxm(p, yaw_to(d), Vec3::new(1.0, 1.0, 1.0)));
                        }
                    }
                }
            }
        }
        // the door's stair tower: a window lit every storey above the door, up
        // to the arrival's height, on the face the door is set in
        let tower = basic_mat!({ color: 0xa9d6b2 });
        let t = parts.part(&geo, &tower);
        let (ex, ez) = plan.exit;
        let (wx, wz) = (ex + DIRS[plan.exit_side][0], ez + DIRS[plan.exit_side][1]);
        let floor = self.floor_y_of(ex, ez);
        let top = self.floor_y_of(plan.spawn.0, plan.spawn.1);
        for y in crate::level::building::door_lights(floor, top).into_iter().skip(1) {
            let p = self.face_point(wx, wz, plan.exit_side ^ 1, 0.0, 0.02, y);
            parts.put(t, boxm(p, yaw_to(plan.exit_side ^ 1), Vec3::new(0.8, 1.0, 1.0)));
        }
    }

    /// The storey's number painted on the wall by the flights and the
    /// platforms where ways meet (the Building counts its storeys in the
    /// thousands, and they count down as you go down)
    fn dress_numbers(&mut self, plan: &BuildingPlan, r: &mut SplitMix64) {
        let base = storey_base(plan.style);
        for nd in &plan.nodes {
            if nd.degree < 3 && r.next() > 0.3 {
                continue;
            }
            // a face of building beside the platform
            let mut spot = None;
            'f: for dz in 0..2 {
                for dx in 0..2 {
                    for d in 0..4 {
                        let (x, z) = (nd.x + dx, nd.z + dz);
                        if self.dress_ground(plan, x + DIRS[d][0], z + DIRS[d][1]).kind == GROUND_MASS
                            && self.in_grid(x + DIRS[d][0], z + DIRS[d][1])
                        {
                            spot = Some((x, z, d));
                            break 'f;
                        }
                    }
                }
            }
            let Some((x, z, d)) = spot else { continue };
            let n = base + nd.storey;
            let h = self.floor_y_of(x, z);
            let (mx, mz) = (x + DIRS[d][0], z + DIRS[d][1]);
            let along = r.range(-0.8, 0.8);
            // (none in a room's doorway, nor over a face that stops under it:
            // its draw is kept)
            if self.off_face(plan, mx, mz, d ^ 1, h + 2.35 - 0.375, h + 2.35 + 0.375) {
                continue;
            }
            let tex = text_tex(
                &format!("{n}"),
                &TextTexOpts {
                    w: 256.0,
                    h: 128.0,
                    font: "bold 96px Helvetica, Arial, sans-serif".into(),
                    color: "#b9b2a0".into(),
                    glow: 0.0,
                    rot: 0.0,
                },
            );
            let m = mesh(plane_geo!(1.5, 0.75), lambert!({ map: tex, transparent: true, opacity: 0.75 }));
            m.position().copy(&self.face_point(mx, mz, d ^ 1, along, 0.01, h + 2.35));
            m.rotation().y = yaw_to(d ^ 1);
            m.user().static_prop = true;
            self.world.add(&m);
        }
    }

    /// Loudspeakers on the walls over a few platforms where ways meet (the
    /// siren comes from them)
    fn dress_speakers(&mut self, plan: &BuildingPlan, parts: &mut Parts, r: &mut SplitMix64) {
        let horn_geo = cyl_geo!(0.07, 0.22, 0.42, 10, 1, true);
        let horn_mat = lambert!({ color: 0x5a5e58, side: DoubleSide });
        let horn = parts.part(&horn_geo, &horn_mat);
        let unit = self.unit_box_geo.clone();
        let bracket = parts.part(&unit, &self.mats.metal);
        self.building_speakers.clear();
        let mut nodes: Vec<_> = plan.nodes.iter().filter(|n| n.degree >= 3).collect();
        // a spread of them: every other one
        nodes.retain(|_| r.next() < 0.55);
        for nd in nodes {
            // the first face of building beside the platform that holds it
            // (not one that stops under it, as a booth does, nor a room's
            // doorway: no draw, so the speakers stay the siren's)
            let mut spot = None;
            'f: for dz in 0..2 {
                for dx in 0..2 {
                    for d in 0..4 {
                        let (x, z) = (nd.x + dx, nd.z + dz);
                        let (mx, mz) = (x + DIRS[d][0], z + DIRS[d][1]);
                        let h = self.floor_y_of(x, z);
                        if self.dress_ground(plan, mx, mz).kind == GROUND_MASS
                            && self.in_grid(mx, mz)
                            && !self.off_face(plan, mx, mz, d ^ 1, h + 3.4, h + 4.0)
                        {
                            spot = Some((x, z, d));
                            break 'f;
                        }
                    }
                }
            }
            let Some((x, z, d)) = spot else { continue };
            let h = self.floor_y_of(x, z);
            let (mx, mz) = (x + DIRS[d][0], z + DIRS[d][1]);
            let yaw = yaw_to(d ^ 1);
            let at = self.face_point(mx, mz, d ^ 1, 1.2, 0.3, h + 3.7);
            parts.put(bracket, boxm(self.face_point(mx, mz, d ^ 1, 1.2, 0.15, h + 3.7), yaw, Vec3::new(0.06, 0.06, 0.3)));
            // the horn looks out and a little down
            let mut q = Quat::from_axis_angle(&Vec3::new(0.0, 1.0, 0.0), yaw);
            q.multiply(&Quat::from_axis_angle(&Vec3::new(1.0, 0.0, 0.0), PI / 2.0 + 0.35));
            parts.put(horn, Mat4::compose(&at, &q, &Vec3::new(1.0, 1.0, 1.0)));
            self.building_speakers.push(at);
        }
    }

    /// The Stair Halls: chains out of the dark, flights nobody can reach
    fn dress_halls(&mut self, plan: &BuildingPlan, parts: &mut Parts, r: &mut SplitMix64) {
        let iron = lambert!({ color: 0x2e2c29 });
        let chain_geo = cyl_geo!(0.05, 0.05, 1.0, 5);
        let chain = parts.part(&chain_geo, &iron);
        let ring_geo = torus_geo!(0.22, 0.04, 5, 12);
        let ring = parts.part(&ring_geo, &iron);
        let voids: Vec<(i32, i32)> = (0..self.h)
            .flat_map(|z| (0..self.w).map(move |x| (x, z)))
            .filter(|&(x, z)| self.dress_ground(plan, x, z).kind == GROUND_VOID && !plan.view_line.contains(&(x, z)))
            .collect();
        if voids.is_empty() {
            return;
        }
        for _ in 0..self.w * self.h / TILES_PER_CHAIN {
            let (x, z) = voids[(r.next() * voids.len() as f64) as usize % voids.len()];
            let (cx, cz) = (self.wx(x) + r.range(-1.5, 1.5), self.wz(z) + r.range(-1.5, 1.5));
            let bottom = r.range(-12.0, 24.0);
            let len = MASS_TOP as f64 - bottom;
            parts.put(chain, Mat4::compose(&Vec3::new(cx, bottom + len / 2.0, cz), &Quat::IDENTITY, &Vec3::new(1.0, len, 1.0)));
            parts.put(ring, boxm(Vec3::new(cx, bottom - 0.2, cz), r.range(0.0, PI), Vec3::new(1.0, 1.0, 1.0)));
        }
        // flights crossing the void that no walk reaches: no parapet on them
        // (as many as on the larger floors: they are where the figure stands)
        let stone = self.arch_mat("floor:stone:2");
        let unit = self.unit_box_geo.clone();
        let step = parts.part(&unit, &stone);
        let mut made = 0;
        for _ in 0..300 {
            if made >= 9 {
                break;
            }
            let (x, z) = voids[(r.next() * voids.len() as f64) as usize % voids.len()];
            let d = (r.next() * 4.0) as usize % 4;
            let len = 3 + (r.next() * 4.0) as i32;
            if !(0..len).all(|k| {
                self.dress_ground(plan, x + DIRS[d][0] * k, z + DIRS[d][1] * k).kind == GROUND_VOID
                    && !plan.view_line.contains(&(x + DIRS[d][0] * k, z + DIRS[d][1] * k))
            }) {
                continue;
            }
            let y0 = (r.range(-8.0, 26.0) / STOREY).round() * STOREY + 1.5;
            let n = (len as f64 * 13.0) as usize;
            let (fx, fz) = (DIRS[d][0] as f64, DIRS[d][1] as f64);
            let yaw = if fx != 0.0 { PI / 2.0 } else { 0.0 };
            for s in 0..n {
                let t = s as f64 / n as f64;
                let p = Vec3::new(
                    self.wx(x) + fx * (t * len as f64 * CELL - CELL / 2.0),
                    y0 + flight_rise(s, n) * len as f64 * STOREY,
                    self.wz(z) + fz * (t * len as f64 * CELL - CELL / 2.0),
                );
                parts.put(step, boxm(p, yaw, Vec3::new(2.4, 0.4, CELL / 13.0 + 0.02)));
                // where the figure across the void may stand
                // (`world::far_figures`): the middle of the landing, level
                // under both feet whichever way it turns
                if s == n / 2 + LANDING / 2 {
                    let at = Vec3::new(p.x, p.y + 0.2, p.z);
                    self.figure_perches.push(at);
                }
            }
            made += 1;
        }
    }

    /// The Works: a ceiling far up with its rows of dead fittings, the
    /// gantries under it, pipes along the walls
    /// The Works' ceiling: 20 m over its highest walk
    pub(crate) fn works_ceiling(&self) -> f64 {
        self.ground.iter().filter(|g| g.walkable()).map(|g| g.h0.max(g.h1) as f64).fold(0.0, f64::max) + 20.0
    }

    fn dress_works(&mut self, plan: &BuildingPlan, parts: &mut Parts, r: &mut SplitMix64) {
        let ceil = self.works_ceiling();
        let (w, h) = (self.w as f64 * CELL, self.h as f64 * CELL);
        // ribbed precast slabs: one mesh, so its map repeats by the slab's own size
        let ribs = self.surfaces.ceil("ribbed", 2).map(|t| {
            let t = t.duplicate();
            let s = crate::visuals::photo::photo_scale("ribbed").unwrap_or([4.0, 1.6]);
            t.set_repeat(w / s[0], h / s[1]);
            t
        });
        let ceil_mat = lambert!({ map: ribs, side: DoubleSide });
        if let Some(t) = self.surfaces.photo_tint("ribbed", 2) {
            ceil_mat.m().color.set_rgb(t, t, t);
        }
        let slab = mesh(box_geo!(w, 1.2, h), ceil_mat);
        slab.position().y = ceil + 0.6;
        slab.user().static_prop = true;
        self.world.add(&slab);
        let unit = self.unit_box_geo.clone();
        let dead = lambert!({ color: 0x3c3d3a });
        let alive = basic_mat!({ color: 0xc8c0a0 });
        let fit_dead = parts.part(&unit, &dead);
        let fit_lit = parts.part(&unit, &alive);
        let mut z = 2;
        while z < self.h - 2 {
            let mut x = 2;
            while x < self.w - 2 {
                let lit = r.next() < 0.06;
                let p = Vec3::new(self.wx(x), ceil - 0.15, self.wz(z));
                parts.put(if lit { fit_lit } else { fit_dead }, boxm(p, 0.0, Vec3::new(2.4, 0.18, 0.5)));
                x += 4;
            }
            z += 4;
        }
        // gantries under the ceiling, across the whole floor
        let steel = lambert!({ color: 0x5c5a4e });
        let beam = parts.part(&unit, &steel);
        for k in 0..3 {
            let zz = self.wz((self.h as f64 * (0.25 + 0.25 * k as f64)) as i32) + r.range(-3.0, 3.0);
            parts.put(beam, boxm(Vec3::new(0.0, ceil - 4.0, zz), 0.0, Vec3::new(w, 1.1, 0.7)));
            parts.put(beam, boxm(Vec3::new(0.0, ceil - 4.0, zz + 6.0), 0.0, Vec3::new(w, 1.1, 0.7)));
            let mut gx = -w / 2.0 + 10.0;
            while gx < w / 2.0 - 10.0 {
                let at = Vec3::new(gx, ceil - 3.45, zz);
                self.figure_perches.push(at);
                gx += 18.0;
            }
            // the trolley, and its hook let down
            let tx = r.range(-w / 3.0, w / 3.0);
            parts.put(beam, boxm(Vec3::new(tx, ceil - 4.9, zz + 3.0), 0.0, Vec3::new(2.6, 0.9, 6.8)));
            let drop = r.range(10.0, 26.0);
            parts.put(beam, boxm(Vec3::new(tx, ceil - 5.3 - drop / 2.0, zz + 3.0), 0.0, Vec3::new(0.05, drop, 0.05)));
            parts.put(beam, boxm(Vec3::new(tx, ceil - 5.5 - drop, zz + 3.0), 0.0, Vec3::new(0.6, 0.6, 0.4)));
        }
        // pipes along the faces of the machines by the walks
        let pipe_geo = cyl_geo!(0.16, 0.16, 1.0, 8);
        let pipe = parts.part(&pipe_geo, &self.mats.metal);
        for zz in 0..self.h {
            for xx in 0..self.w {
                let g = self.dress_ground(plan, xx, zz);
                if !g.walkable() || r.next() > 0.5 {
                    continue;
                }
                for d in 0..4 {
                    let (mx, mz) = (xx + DIRS[d][0], zz + DIRS[d][1]);
                    if self.dress_ground(plan, mx, mz).kind != GROUND_MASS {
                        continue;
                    }
                    let along = if DIRS[d][0] != 0 { Vec3::new(0.0, 0.0, 1.0) } else { Vec3::new(1.0, 0.0, 0.0) };
                    let q = Quat::from_axis_angle(&Vec3::new(along.z, 0.0, -along.x), PI / 2.0);
                    for (k, y) in [2.9, 3.3].iter().enumerate() {
                        let y = g.h0 as f64 + y;
                        // (none across a room's doorway, nor over a face that
                        // stops under it)
                        if self.off_face(plan, mx, mz, d ^ 1, y - 0.16, y + 0.16) {
                            continue;
                        }
                        let p = self.face_point(xx, zz, d, 0.0, -0.25 - k as f64 * 0.38, y);
                        parts.put(pipe, Mat4::compose(&p, &q, &Vec3::new(1.0, CELL, 1.0)));
                    }
                }
            }
        }
    }
}

/// How many steps of a flight that crosses the void are its landing,
/// halfway up: level, as a long flight's landing is, so the figure that
/// stands there is level under both feet whichever way it turns (on a step
/// it stood with a foot in the step above when it turned up the flight)
pub(crate) const LANDING: usize = 4;

/// How far up its rise step `s` of a flight of `n` stands (0 at the foot):
/// evenly, but for the landing from step `n / 2`, which keeps that step's
/// height for `LANDING` steps; the steps after it go on rising from there,
/// so the flight ends `LANDING - 1` steps short of its full rise (its last
/// step at `(n - LANDING) / n`, not `(n - 1) / n`). That is only for the
/// flights that cross the Stair Halls' void (`dress_halls`): no walk reaches
/// either end of one, so neither end has a floor to meet
pub(crate) fn flight_rise(s: usize, n: usize) -> f64 {
    let land = n / 2;
    let held = s.saturating_sub(land).min(LANDING - 1);
    (s - held) as f64 / n as f64
}

#[cfg(test)]
mod flight_tests {
    use super::*;

    #[test]
    fn a_flight_has_a_level_landing_where_the_figure_stands() {
        for len in 3..=6 {
            let n = len * 13;
            let ys: Vec<f64> = (0..n).map(|s| flight_rise(s, n)).collect();
            let land = n / 2;
            // level over the landing, and the figure in its middle with a
            // step's depth of level floor either side of it
            for s in land..land + LANDING {
                assert_eq!(ys[s], ys[land], "len {len} step {s}");
            }
            let at = land + LANDING / 2;
            assert!(at > land && at + 1 < land + LANDING, "len {len}: the figure stands within the landing");
            // rising everywhere else, never falling
            for s in 1..n {
                assert!(ys[s] >= ys[s - 1], "len {len} step {s}");
                if !(land + 1..land + LANDING).contains(&s) {
                    assert!(ys[s] > ys[s - 1], "len {len} step {s} should rise");
                }
            }
        }
    }
}
