//! The vending machine's body (`docs/design/VENDING.md`): a glass-front spiral
//! snack machine at its real size. The cabinet, door, trays, coils and
//! hardware are the Blender parts of family `vending_machine`, placed as modelled; the
//! goods, their labels and the glass are made here. Everything that does not
//! move is merged by material into one mesh each, so a machine is about
//! twenty draws however many coils and goods it holds; a coil that turns
//! is taken out of the merge for as long as it moves (`Body::lift`).
//!
//! The machine's frame: x across its face (to your right as you face it), y
//! up from the floor, z out of its face toward you; the floor centre of its
//! footprint is the origin.
use crate::assets::props::prop_part_as_modelled;
use crate::prelude::*;
use std::collections::BTreeMap;

/// The cabinet: 1.83 m tall, 0.99 m wide, 0.89 m deep (72 × 39 × 35 in)
pub const W: f64 = 0.99;
pub const H: f64 = 1.83;
pub const D: f64 = 0.889;
/// The door's thickness, and the kick plate's height under it
pub const DOOR_T: f64 = 0.075;
pub const KICK: f64 = 0.10;
/// z of the door's face
pub const FACE: f64 = D / 2.0;
/// The window: x0, x1, y0, y1
pub const WIN: [f64; 4] = [-0.455, 0.205, 0.600, 1.735];
/// The selection column: x0, x1
pub const COL: [f64; 2] = [0.232, 0.470];
/// The delivery opening: x0, x1, y0, y1
pub const BIN: [f64; 4] = [-0.430, 0.180, 0.200, 0.470];
/// z of the glass (set back in the door)
pub const GLASS_Z: f64 = FACE - 0.026;
/// The product cabinet behind the window: centre x, y, z
pub const CAB: [f64; 3] = [-0.125, 1.10, FACE - DOOR_T - 0.36];
/// The trays' pans, top ("A") first: y of each pan's underside
pub const TRAY_Y: [f64; 6] = [1.49, 1.30, 1.11, 0.92, 0.73, 0.54];
/// z of the trays' centres (their price channels just behind the glass)
pub const TRAY_Z: f64 = -0.01;
/// A tray's width, and z of the coils' centres
pub const TRAY_W: f64 = 0.672;
pub const COIL_Z: f64 = 0.010;
/// A coil's length, front to back
pub const COIL_LEN: f64 = 0.600;
/// The selection column's panel: its top edge
pub const PANEL_TOP: f64 = WIN[3] + 0.005;
/// The panel's face
pub const PANEL_FACE: f64 = FACE + 0.002;

/// What a piece is drawn with: one material each, one merged mesh each.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Slot {
    /// the painted steel of the cabinet and door
    Paint,
    /// moulded charcoal plastic: the panel, the validator, the keys, the kick plate
    Trim,
    /// chrome and plated steel: the coin entry, the return lever, the lock, the feet
    Metal,
    /// the zinc-plated coils
    Coil,
    /// galvanised trays and the vend motors' housings
    Tray,
    /// the white liner of the product cabinet
    Liner,
    /// what is for sale (one atlas of wrappers)
    Goods,
    /// the plastic of the almond water's bottles
    Bottle,
    /// the price strips (one atlas)
    Labels,
    /// the fluorescent tubes
    Tube,
    /// the glass
    Glass,
    /// the delivery flap (smoked polycarbonate)
    Flap,
    /// the stickers and tape on the door
    Decal,
    /// the keypad's keys (pale plastic) and their legends
    Keys,
    Legend,
    /// rust and scuffs along the door's bottom
    Rust,
}

/// One placed thing: its geometry (in its own frame), where it stands in the
/// machine, what it is drawn with, the coil it belongs to (it moves with the
/// coil), and for a textured one its cell of the slot's atlas (u0, v0, du,
/// dv: its UVs 0..1 land there).
#[derive(Clone)]
pub struct Piece {
    pub geo: Geo,
    pub m: Mat4,
    pub slot: Slot,
    pub coil: Option<usize>,
    pub cell: Option<[f64; 4]>,
    /// the good it is a part of (a bottle is three pieces), if any
    pub item: Option<usize>,
}

/// Blender part `name` (family `vending_machine`) as modelled; a small box when the part
/// is missing (a build without the batch).
pub fn part(name: &str) -> Geo {
    prop_part_as_modelled(name).unwrap_or_else(|| box_geometry(0.02, 0.02, 0.02, 1.0, 1.0, 1.0))
}

/// A placement at (x, y, z)
pub fn at(x: f64, y: f64, z: f64) -> Mat4 {
    let mut m = Mat4::IDENTITY;
    m.make_translation(x, y, z);
    m
}
/// A placement at (x, y, z), turned by Euler angles (XYZ)
pub fn at_rot(x: f64, y: f64, z: f64, rx: f64, ry: f64, rz: f64) -> Mat4 {
    let q = Quat::from_euler(&Euler::new(rx, ry, rz));
    Mat4::compose(&Vec3::new(x, y, z), &q, &Vec3::new(1.0, 1.0, 1.0))
}

/// The pieces merged into one geometry: positions and normals carried by
/// each piece's placement, UVs moved into each piece's atlas cell.
pub fn merge<'a>(pieces: impl Iterator<Item = &'a Piece>) -> Geo {
    let (mut pos, mut nrm, mut uv, mut idx) = (vec![], vec![], vec![], vec![]);
    for p in pieces {
        let g = p.geo.r();
        let base = (pos.len() / 3) as u32;
        let n = g.position.len() / 3;
        for i in 0..n {
            let mut v = Vec3::new(g.position[i * 3] as f64, g.position[i * 3 + 1] as f64, g.position[i * 3 + 2] as f64);
            v.apply_matrix4(&p.m);
            pos.extend_from_slice(&[v.x as f32, v.y as f32, v.z as f32]);
            let mut k = if g.normal.len() >= (i + 1) * 3 {
                Vec3::new(g.normal[i * 3] as f64, g.normal[i * 3 + 1] as f64, g.normal[i * 3 + 2] as f64)
            } else {
                Vec3::new(0.0, 1.0, 0.0)
            };
            k.transform_direction(&p.m);
            nrm.extend_from_slice(&[k.x as f32, k.y as f32, k.z as f32]);
            let (u, w) = if g.uv.len() >= (i + 1) * 2 { (g.uv[i * 2] as f64, g.uv[i * 2 + 1] as f64) } else { (0.0, 0.0) };
            let (u, w) = match p.cell {
                Some(c) => (c[0] + u.clamp(0.0, 1.0) * c[2], c[1] + w.clamp(0.0, 1.0) * c[3]),
                None => (u, w),
            };
            uv.extend_from_slice(&[u as f32, w as f32]);
        }
        match &g.index {
            Some(ix) => idx.extend(ix.iter().map(|&i| i + base)),
            None => idx.extend((0..n as u32).map(|i| i + base)),
        }
    }
    let geo = Geo::new();
    geo.set_position(pos).set_normal(nrm).set_uv(uv).set_index(idx);
    geo.compute_bounding_box();
    geo.compute_bounding_sphere();
    geo
}

/// The machine's drawn body: its pieces, a material and a merged mesh per
/// slot under `root`, and the pieces of any coil lifted out of the merge.
pub struct Body {
    pub root: Obj,
    pub pieces: Vec<Piece>,
    pub mats: BTreeMap<Slot, Mat>,
    pub meshes: BTreeMap<Slot, Obj>,
    /// the coil lifted out of the merge (it is turning), and its meshes
    pub lifted: Option<(usize, Vec<(usize, Obj)>)>,
}

impl Body {
    /// One mesh per slot of every piece but those of coil `except`.
    pub fn draw(&mut self, except: Option<usize>) {
        for (_, o) in std::mem::take(&mut self.meshes) {
            self.root.remove(&o);
        }
        let slots: Vec<Slot> = self.mats.keys().copied().collect();
        for s in slots {
            let geo = merge(self.pieces.iter().filter(|p| p.slot == s && (except.is_none() || p.coil != except)));
            if geo.r().position.is_empty() {
                continue;
            }
            let o = mesh(geo, &self.mats[&s]);
            if matches!(s, Slot::Glass | Slot::Flap | Slot::Decal | Slot::Legend | Slot::Rust) {
                o.set_render_order(2);
            }
            self.root.add(&o);
            self.meshes.insert(s, o);
        }
    }

    /// Take coil `k`'s pieces out of the merge, each its own mesh placed as
    /// it stood (so they can be moved); returns them, by piece index.
    pub fn lift(&mut self, k: usize) -> Vec<(usize, Obj)> {
        self.settle();
        self.draw(Some(k));
        let mut out = vec![];
        for (i, p) in self.pieces.iter().enumerate() {
            if p.coil != Some(k) {
                continue;
            }
            let one = Piece { m: Mat4::IDENTITY, ..p.clone() };
            let o = mesh(merge(std::iter::once(&one)), &self.mats[&p.slot]);
            let (pos, q, s) = p.m.decompose();
            o.set_pos_v(&pos);
            o.set_quat(&q);
            o.set_scale(s.x, s.y, s.z);
            self.root.add(&o);
            out.push((i, o.clone()));
        }
        self.lifted = Some((k, out.clone()));
        out
    }

    /// Put a lifted coil back into the merge, its pieces where they now stand.
    pub fn settle(&mut self) {
        let Some((_, list)) = self.lifted.take() else { return };
        for (i, o) in list {
            o.update_matrix();
            self.pieces[i].m = o.matrix();
            self.root.remove(&o);
        }
        self.draw(None);
    }
}

/// One coil's place: its code ("B3"), its tray, its x, its spiral (chips or
/// candy), and how many turns it has (one good between each pair).
#[derive(Clone, Debug)]
pub struct CoilPlace {
    pub code: String,
    pub tray: usize,
    pub x: f64,
    pub wide: bool,
    pub turns: usize,
}
impl CoilPlace {
    /// The spiral's radius (to the wire's middle) and wire radius
    pub fn radius(&self) -> (f64, f64) {
        match (self.wide, self.turns) {
            (false, _) => (0.032, 0.0022),
            (true, 7) => (0.050, 0.0026),
            _ => (0.050, 0.0025),
        }
    }
    /// y of the coil's axis
    pub fn axis_y(&self) -> f64 {
        let (r, w) = self.radius();
        TRAY_Y[self.tray] + 0.010 + r + w
    }
    /// The pitch of its turns, m
    pub fn pitch(&self) -> f64 {
        COIL_LEN / self.turns as f64
    }
    /// z of the place between turns `k` and `k + 1`, counted from the front
    /// (0: the front place, the one that drops next)
    pub fn place_z(&self, k: usize) -> f64 {
        COIL_Z + COIL_LEN / 2.0 - self.pitch() * (k as f64 + 0.5)
    }
}

/// The trays and their coils, top first: three trays of five wide spirals
/// (bags), two of eight narrow ones (bars, packets, batteries), and one of
/// five wide spirals of seven turns (pastries and bottles).
pub fn coil_places() -> Vec<CoilPlace> {
    let mut out = vec![];
    for (t, letter) in ["A", "B", "C", "D", "E", "F"].iter().enumerate() {
        let (n, wide, turns) = match t {
            3 | 4 => (8, false, 15),
            5 => (5, true, 7),
            _ => (5, true, 10),
        };
        let pitch = TRAY_W / n as f64;
        for i in 0..n {
            out.push(CoilPlace {
                code: format!("{letter}{}", i + 1),
                tray: t,
                x: CAB[0] - TRAY_W / 2.0 + pitch * (i as f64 + 0.5),
                wide,
                turns,
            });
        }
    }
    out
}

/// The keypad's keys, as they read on the machine: three across, seven
/// down (A to H and J, there is no I; the digits; * and #)
pub const KEYS: [&str; 21] = ["A", "B", "C", "D", "E", "F", "G", "H", "J", "1", "2", "3", "4", "5", "6", "7", "8", "9", "*", "0", "#"];
/// Where key `i` stands (x, y), on the panel's face
pub fn key_xy(i: usize) -> (f64, f64) {
    let (col, row) = ((i % 3) as f64, (i / 3) as f64);
    let cx = (COL[0] + COL[1]) / 2.0;
    (cx - 0.024 + col * 0.024, PANEL_TOP - 0.142 - row * 0.0215)
}
/// The coin entry, the coin return's button, the bill validator and the
/// coin return cup: their centres (x, y)
pub fn coin_entry_xy() -> (f64, f64) {
    ((COL[0] + COL[1]) / 2.0 - 0.045, PANEL_TOP - 0.445)
}
pub fn return_lever_xy() -> (f64, f64) {
    ((COL[0] + COL[1]) / 2.0 + 0.040, PANEL_TOP - 0.425)
}
pub fn validator_xy() -> (f64, f64) {
    ((COL[0] + COL[1]) / 2.0, PANEL_TOP - 0.585)
}
pub fn coin_cup_xy() -> (f64, f64) {
    ((COL[0] + COL[1]) / 2.0, PANEL_TOP - 0.760)
}
/// The display's window: centre (x, y), size (w, h)
pub fn display_rect() -> (f64, f64, f64, f64) {
    ((COL[0] + COL[1]) / 2.0, PANEL_TOP - 0.070, 0.164, 0.046)
}

/// The machine's fixed pieces: the cabinet, door, window frame, liner,
/// trays with their coils and motors, the selection column's hardware,
/// the delivery bin and flap, the kick plate, the feet and the lock. The
/// coils carry their index in `places`.
pub fn hardware(places: &[CoilPlace]) -> Vec<Piece> {
    let mut v = vec![];
    let mut put =
        |name: &str, m: Mat4, slot: Slot, coil: Option<usize>| v.push(Piece { geo: part(name), m, slot, coil, cell: None, item: None });
    put("Vending__cabinet", at(0.0, 0.02 + (H - 0.02) / 2.0, -DOOR_T / 2.0), Slot::Paint, None);
    put("Vending__door", at(0.0, KICK + (H - KICK) / 2.0, FACE - DOOR_T / 2.0), Slot::Paint, None);
    put("Vending__glassFrame", at((WIN[0] + WIN[1]) / 2.0, (WIN[2] + WIN[3]) / 2.0, FACE - 0.008), Slot::Trim, None);
    put("Vending__liner", at(CAB[0], CAB[1], CAB[2]), Slot::Liner, None);
    for (t, &y) in TRAY_Y.iter().enumerate() {
        let _ = t;
        put("Vending__tray", at(CAB[0], y, TRAY_Z), Slot::Tray, None);
    }
    for (k, c) in places.iter().enumerate() {
        let name = match (c.wide, c.turns) {
            (false, _) => "Vending__coilCandy",
            (true, 7) => "Vending__coilBottle",
            _ => "Vending__coilChip",
        };
        put(name, at(c.x, c.axis_y(), COIL_Z), Slot::Coil, Some(k));
        put("Vending__motor", at(c.x, TRAY_Y[c.tray] + 0.045, COIL_Z - COIL_LEN / 2.0 - 0.034), Slot::Tray, None);
    }
    let cx = (COL[0] + COL[1]) / 2.0;
    put("Vending__panel", at(cx, (WIN[2] + WIN[3]) / 2.0, FACE - 0.005), Slot::Trim, None);
    for i in 0..KEYS.len() {
        let (x, y) = key_xy(i);
        put("Vending__key", at(x, y, PANEL_FACE - 0.006), Slot::Keys, None);
    }
    let (x, y) = coin_entry_xy();
    put("Vending__coinEntry", at(x, y, PANEL_FACE - 0.004), Slot::Metal, None);
    let (x, y) = return_lever_xy();
    put("Vending__returnLever", at(x, y, PANEL_FACE + 0.001), Slot::Metal, None);
    let (x, y) = validator_xy();
    put("Vending__validator", at(x, y, PANEL_FACE - 0.012), Slot::Trim, None);
    let (x, y) = coin_cup_xy();
    put("Vending__coinCup", at(x, y, PANEL_FACE - 0.020), Slot::Trim, None);
    let (bx, by) = ((BIN[0] + BIN[1]) / 2.0, (BIN[2] + BIN[3]) / 2.0);
    put("Vending__bin", at(bx, by, FACE - DOOR_T - 0.105), Slot::Trim, None);
    put("Vending__kick", at(0.0, 0.02 + (KICK - 0.02) / 2.0, FACE - 0.03), Slot::Trim, None);
    for (sx, sz) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
        put("Vending__foot", at(sx * (W / 2.0 - 0.06), 0.010, sz * (D / 2.0 - 0.08)), Slot::Metal, None);
    }
    put("Vending__lock", at(COL[1] - 0.040, 0.80, FACE + 0.007), Slot::Metal, None);
    // the tube: across the top, inside the door, behind the window's top edge
    let (tx, ty, tz) = tube_xyz();
    put("Vending__tube", at_rot(tx, ty, tz, 0.0, 0.0, PI / 2.0), Slot::Tube, None);
    // its data plate, riveted to its back high on one side (seen when it
    // lies on its face): bare aluminium, 110 x 70 mm
    let plate = box_geometry(0.110, 0.070, 0.0015, 1.0, 1.0, 1.0);
    v.push(Piece { geo: plate, m: at(-0.26, 1.48, -D / 2.0 - 0.0008), slot: Slot::Metal, coil: None, cell: None, item: None });
    v
}

/// The tube's middle (it lies along x)
pub fn tube_xyz() -> (f64, f64, f64) {
    (CAB[0], WIN[3] - 0.022, FACE - DOOR_T - 0.032)
}

/// The flap of the delivery bin (its own mesh: it swings in when pushed),
/// hinged along its top edge: its pivot (y, z) and the part.
pub fn flap_pivot() -> (f64, f64, f64) {
    ((BIN[0] + BIN[1]) / 2.0, BIN[3] - 0.010, FACE - 0.018)
}
