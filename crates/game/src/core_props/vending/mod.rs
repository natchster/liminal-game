//! The vending machine (`docs/design/VENDING.md`): a glass-front spiral snack
//! machine against a wall, one on most floors of the rooms, still on or
//! long dead. Its body is `model` (Blender parts of family `vending_machine`, merged by
//! material), its goods `goods`, what is printed on it `paint`, and what
//! it does `rules`.
//!
//! Built in the level's build like any prop (its choices from the build's
//! stream); everything it does once the level runs draws from the native
//! stream. Its state lives in `Game::vending`.
pub mod goods;
pub mod model;
pub mod paint;
pub mod rules;

use crate::core_props::{Built, PropsCtx};
use crate::prelude::*;
use crate::visuals::textures::{handwritten_tex, HandOpts};
use model::*;
use paint::*;

/// The painted steel a machine came in (sRGB), after the makers' own names:
/// True Grit Black (twice as common), Dark Gray, Dove Gray, a dark brown
const PAINTS: [u32; 5] = [0x1c1c1d, 0x1c1c1d, 0x3b3c3e, 0x8c8d8a, 0x34291f];

/// What a coil holds: a design of the goods' atlas and how many are left
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stock {
    pub design: usize,
    pub count: usize,
}

/// A vending machine of the level (see `rules` for what it does)
pub struct Machine {
    /// where it stands (its floor centre, its yaw); `tilt` turns about its
    /// front bottom edge (its lean, its rocking, its fall)
    pub root: Obj,
    pub tilt: Obj,
    pub body: Body,
    pub places: Vec<CoilPlace>,
    pub stock: Vec<Stock>,
    /// still on: lit, humming, its display awake
    pub powered: bool,
    /// the display: its mesh, what it is painted on, its texture
    pub display: Obj,
    pub display_canvas: Canvas,
    pub display_tex: Tex,
    /// the goods each coil holds, front first (the ids of their pieces)
    pub items: Vec<Vec<usize>>,
    /// the flap of the delivery bin (it swings in when pushed)
    pub flap: Obj,
    /// its tubes and its lights (they go out with it), and each light's power
    pub tubes: Mat,
    pub lights: Vec<Obj>,
    pub light_power: Vec<f64>,
    /// its inside's materials and their lit colour and glow (kept the first
    /// time it goes dark, rule 1)
    pub inside: Vec<(Mat, Color, Color)>,
    /// a hand on the inside of its glass (a dead one's, rule 9), hidden
    pub print: Option<Obj>,
    /// its own stream's seed (what it restocks with), and how full each
    /// coil was built
    pub seed: u32,
    pub full: Vec<usize>,
    /// its hum (made when it is first heard; `rules::vending_hum`)
    pub hum: Option<crate::audio::soundscape::MachineHum>,
    /// what it is doing (`rules`)
    pub state: rules::State,
}

impl Machine {
    /// Where it stands: its floor centre (the level's settling may have
    /// moved it onto its wall's face, and turned it with a leaning wall)
    pub fn pos(&self) -> Vec3 {
        self.root.pos()
    }
    /// The way its face looks (its turn about y)
    pub fn yaw(&self) -> f64 {
        self.root.rot().y
    }
    /// A point in the machine's frame, in the world
    pub fn world(&self, x: f64, y: f64, z: f64) -> Vec3 {
        let (p, (s, c)) = (self.pos(), self.yaw().sin_cos());
        Vec3::new(p.x + x * c + z * s, p.y + y, p.z - x * s + z * c)
    }
    /// Fill coil `k` again (rule 8): as full as it was built, its goods
    /// made anew from `seed` (a stream of its own)
    pub fn restock(&mut self, k: usize, seed: u32) {
        let c = self.places[k].clone();
        let s = Stock { design: self.stock[k].design, count: self.full[k] };
        self.body.pieces.retain(|p| !(p.coil == Some(k) && p.item.is_some()));
        let mut next = self.body.pieces.iter().filter_map(|p| p.item).max().map(|x| x + 1).unwrap_or(0).max(10_000 * (k + 1));
        let mut add = vec![];
        let ids =
            crate::base::rng::with_own_stream(seed ^ (k as u32).wrapping_mul(0x85EB_CA6B), || goods_pieces(k, &c, &s, &mut next, &mut add));
        self.body.pieces.extend(add);
        self.items[k] = ids;
        self.stock[k] = s;
    }
    /// Whether piece `pi` is part of coil `k`'s front good
    pub fn stock_front(&self, k: usize, pi: usize) -> bool {
        self.items[k].first().is_some_and(|&id| self.body.pieces[pi].item == Some(id))
    }
    /// The id of coil `k`'s front good
    pub fn front_item(&self, k: usize) -> Option<usize> {
        self.items[k].first().copied()
    }
    /// A world point in the machine's frame
    pub fn local(&self, q: &Vec3) -> Vec3 {
        let (p, (s, c)) = (self.pos(), self.yaw().sin_cos());
        let (dx, dz) = (q.x - p.x, q.z - p.z);
        Vec3::new(dx * c - dz * s, q.y - p.y, dx * s + dz * c)
    }
}

/// What goes in each coil: bags in the top three trays (a design over one
/// or two coils side by side, as a route stocks them), bars in the next two
/// with a card of batteries among them, and in the bottom tray pastries and
/// almond water. A dead machine was left half sold; a coil is sometimes
/// empty in any.
fn stock(places: &[CoilPlace], powered: bool) -> Vec<Stock> {
    let d = designs();
    let of = |k: Ware| -> Vec<usize> { d.iter().enumerate().filter(|(_, x)| x.kind == k).map(|(i, _)| i).collect() };
    let (bags, bars, pastries) = (of(Ware::Bag), of(Ware::Bar), of(Ware::Pastry));
    let almond = of(Ware::Almond)[0];
    let battery = of(Ware::Battery)[0];
    let battery_at = irand(0, 15) as usize;
    let mut out = vec![];
    let mut last = usize::MAX;
    for (i, c) in places.iter().enumerate() {
        let design = match c.tray {
            0..=2 => {
                // the same bag twice in a row now and then
                if last != usize::MAX && rnd() < 0.3 && out.len() % 5 != 0 {
                    last
                } else {
                    choice(&bags)
                }
            }
            3 | 4 => {
                if (i - 15) == battery_at {
                    battery
                } else {
                    choice(&bars)
                }
            }
            _ => {
                if i - 31 >= 3 {
                    almond
                } else {
                    choice(&pastries)
                }
            }
        };
        last = design;
        let full = c.turns;
        let count = if rnd() < if powered { 0.12 } else { 0.35 } {
            0
        } else if powered {
            irand((full / 3).max(1) as i64, full as i64) as usize
        } else {
            irand(1, (full / 2).max(1) as i64) as usize
        };
        out.push(Stock { design, count });
    }
    out
}

/// The goods in coil `k`: `count` of its design between the coil's turns,
/// from the front, each a little askew; the ids given them (front first)
fn goods_pieces(k: usize, c: &CoilPlace, s: &Stock, next: &mut usize, out: &mut Vec<Piece>) -> Vec<usize> {
    let mut ids = vec![];
    let d = designs()[s.design];
    let cell = Some(cell_uv(d.cell));
    let y = TRAY_Y[c.tray] + 0.010;
    // each design its own size, a little
    let k_size = 0.92 + 0.16 * ((s.design * 37 % 11) as f64 / 10.0);
    for j in 0..s.count {
        let id = *next;
        *next += 1;
        ids.push(id);
        let z = c.place_z(j);
        let fine = j < 2;
        // a good now and then turned in its coil, or leaning on the next
        let (jx, mut yaw, mut lean) = (rand2(-0.004, 0.004), rand2(-0.06, 0.06), rand2(-0.03, 0.06));
        if rnd() < 0.15 {
            yaw += rand2(-0.25, 0.25);
        }
        if rnd() < 0.15 {
            lean += rand2(0.05, 0.12);
        }
        match d.kind {
            Ware::Bag => {
                let g = goods::pouch(0.150 * k_size, 0.205 * k_size, rand2(0.040, 0.052), 0.055, rand1(0.8), false, rand1(6.0), fine);
                out.push(Piece {
                    geo: g,
                    m: at_rot(c.x + jx, y, z, lean, yaw, 0.0),
                    slot: Slot::Goods,
                    coil: Some(k),
                    cell,
                    item: Some(id),
                });
            }
            Ware::Bar => {
                let g = goods::pouch(0.058, 0.135 * k_size.min(1.0), 0.020, 0.05, rand1(0.3), true, rand1(6.0), fine);
                out.push(Piece {
                    geo: g,
                    m: at_rot(c.x + jx, y, z, lean, yaw, 0.0),
                    slot: Slot::Goods,
                    coil: Some(k),
                    cell,
                    item: Some(id),
                });
            }
            Ware::Pastry => {
                let g = goods::pouch(0.125, 0.125, 0.034, 0.06, rand1(0.5), false, rand1(6.0), fine);
                out.push(Piece {
                    geo: g,
                    m: at_rot(c.x + jx, y, z, lean, yaw, 0.0),
                    slot: Slot::Goods,
                    coil: Some(k),
                    cell,
                    item: Some(id),
                });
            }
            Ware::Battery => {
                let g = goods::card(0.075, 0.130, 0.020);
                out.push(Piece {
                    geo: g,
                    m: at_rot(c.x + jx, y, z, lean * 0.5, yaw, 0.0),
                    slot: Slot::Goods,
                    coil: Some(k),
                    cell,
                    item: Some(id),
                });
            }
            Ware::Almond => {
                // the bottles found on the floors: nothing printed on them
                let (body, cap) = goods::bottle();
                let m = at_rot(c.x + jx, y, z, 0.0, rand1(PI * 2.0), 0.0);
                out.push(Piece { geo: body, m, slot: Slot::Bottle, coil: Some(k), cell: None, item: Some(id) });
                out.push(Piece { geo: cap, m, slot: Slot::Bottle, coil: Some(k), cell: None, item: Some(id) });
            }
        }
    }
    ids
}

/// The power cord: out of the back's lower right, down to the floor and
/// along it to the wall behind (a flat-sided tube, 8 mm)
fn cord_geo() -> Geo {
    let pts: Vec<Vec3> = [
        (W / 2.0 - 0.10, 0.22, -D / 2.0 + 0.005),
        (W / 2.0 - 0.10, 0.10, -D / 2.0 - 0.010),
        (W / 2.0 - 0.05, 0.006, -D / 2.0 - 0.020),
        (W / 2.0 + 0.04, 0.005, -D / 2.0 - 0.030),
        (W / 2.0 + 0.10, 0.005, -D / 2.0 - 0.045),
    ]
    .iter()
    .map(|&(x, y, z)| Vec3::new(x, y, z))
    .collect();
    let (r, sides) = (0.004, 5usize);
    let (mut pos, mut idx) = (vec![], vec![]);
    for (i, p) in pts.iter().enumerate() {
        let next = pts[(i + 1).min(pts.len() - 1)];
        let prev = pts[i.saturating_sub(1)];
        let mut t = next;
        t.sub(&prev);
        t.normalize();
        let mut a = Vec3::new(0.0, 1.0, 0.0);
        if t.y.abs() > 0.9 {
            a = Vec3::new(1.0, 0.0, 0.0);
        }
        let mut u = t;
        u.cross(&a);
        u.normalize();
        let mut v = t;
        v.cross(&u);
        v.normalize();
        for k in 0..sides {
            let th = k as f64 / sides as f64 * PI * 2.0;
            let (c, s) = (th.cos() * r, th.sin() * r);
            pos.extend_from_slice(&[(p.x + u.x * c + v.x * s) as f32, (p.y + u.y * c + v.y * s) as f32, (p.z + u.z * c + v.z * s) as f32]);
        }
        if i + 1 < pts.len() {
            let (b0, b1) = ((i * sides) as u32, ((i + 1) * sides) as u32);
            for k in 0..sides as u32 {
                let k2 = (k + 1) % sides as u32;
                idx.extend_from_slice(&[b0 + k, b1 + k, b1 + k2, b0 + k, b1 + k2, b0 + k2]);
            }
        }
    }
    let g = Geo::new();
    g.set_position(pos).set_uv(vec![0.0; pts.len() * sides * 2]).set_index(idx);
    g.compute_vertex_normals();
    g
}

/// The front good of coil `c` (id `item`) hangs, tipped against the glass
/// off the spiral's end (as `rules` leaves one)
fn hang_front(body: &mut Body, c: &CoilPlace, item: usize) {
    let end = COIL_Z + COIL_LEN / 2.0;
    let lean = ((GLASS_Z - 0.006 - end) / 0.2).clamp(0.0, 1.0).asin();
    for p in body.pieces.iter_mut().filter(|p| p.item == Some(item)) {
        let (mut pos, q, _) = p.m.decompose();
        pos.z += c.pitch();
        let tip = Quat::from_axis_angle(&Vec3::new(1.0, 0.0, 0.0), lean);
        let mut q2 = tip;
        q2.multiply(&q);
        p.m = Mat4::compose(&pos, &q2, &Vec3::new(1.0, 1.0, 1.0));
    }
}

/// The price strips: under each coil its code and its price
fn strip_pieces(places: &[CoilPlace]) -> (Tex, Vec<Piece>) {
    let mut rows: Vec<Vec<(String, String)>> = vec![vec![]; TRAY_Y.len()];
    for c in places {
        let price = match c.tray {
            0..=2 => "$1.25",
            3 | 4 => "$1.00",
            _ => "$1.50",
        };
        rows[c.tray].push((c.code.clone(), price.to_string()));
    }
    let tex = strips_tex(&rows);
    let n = rows.len() as f64;
    let mut out = vec![];
    for (t, &y) in TRAY_Y.iter().enumerate() {
        let g = plane_geometry(TRAY_W - 0.01, 0.022, 1.0, 1.0);
        // the price channel's face, tipped back 15°
        let m = at_rot(CAB[0], y + 0.013, TRAY_Z + 0.3515, -0.26, 0.0, 0.0);
        let cell = Some([0.0, 1.0 - (t as f64 + 1.0) / n, 1.0, 1.0 / n]);
        out.push(Piece { geo: g, m, slot: Slot::Labels, coil: None, cell, item: None });
    }
    (tex, out)
}

/// The door's stickers (cells of `decal_tex`) and the keys' legends
fn decal_pieces() -> Vec<Piece> {
    let mut out = vec![];
    let mut put = |w: f64, h: f64, x: f64, y: f64, z: f64, rz: f64, slot: Slot, cell: [f64; 4]| {
        let g = plane_geometry(w, h, 1.0, 1.0);
        out.push(Piece { geo: g, m: at_rot(x, y, z, 0.0, 0.0, rz), slot, coil: None, cell: Some(cell), item: None });
    };
    let cx = (COL[0] + COL[1]) / 2.0;
    let z = PANEL_FACE + 0.0015;
    put(0.100, 0.050, cx, PANEL_TOP - 0.525, z, 0.0, Slot::Decal, decal_uv(EXACT));
    put(0.110, 0.020, cx, PANEL_TOP - 0.112, z, 0.0, Slot::Decal, decal_uv(SELECT));
    let (rx, ry) = return_lever_xy();
    put(0.064, 0.012, rx - 0.004, ry - 0.026, z, 0.0, Slot::Decal, decal_uv(RETURN));
    let (vx, vy) = validator_xy();
    put(0.070, 0.026, vx, vy + 0.052, z, 0.0, Slot::Decal, decal_uv(BILLS));
    put(0.120, 0.060, (BIN[0] + BIN[1]) / 2.0, BIN[3] - 0.060, FACE - 0.012, 0.0, Slot::Decal, decal_uv(PUSH));
    put(0.180, 0.045, cx, 0.66, FACE + 0.0015, rand2(-0.05, 0.05), Slot::Decal, decal_uv(SERVICE));
    put(0.210, 0.066, cx, 0.50, FACE + 0.0015, rand2(-0.02, 0.02), Slot::Decal, decal_uv(NEVER));
    for i in 0..KEYS.len() {
        let (x, y) = key_xy(i);
        let cell = [(i % 4) as f64 * 0.25, 1.0 - ((i / 4) as f64 + 1.0) * 0.125, 0.25, 0.125];
        put(0.013, 0.013, x, y, PANEL_FACE - 0.0015, 0.0, Slot::Legend, cell);
    }
    out
}

/// The materials of a machine painted `paint` (sRGB hex), dead or on
fn materials(paint: u32, powered: bool, strips: &Tex) -> std::collections::BTreeMap<Slot, Mat> {
    // inside the cabinet the room's light comes in only through the glass:
    // a dead machine is dim inside, a live one lit by its tubes (and its
    // white liner glows with them)
    let k = if powered { 0.82 } else { 0.30 };
    let dim = |hex: u32| {
        let c = |sh: u32| ((((hex >> sh) & 0xff) as f64 * k).round() as u32) << sh;
        c(16) | c(8) | c(0)
    };
    let mut m = std::collections::BTreeMap::new();
    m.insert(Slot::Paint, phong!({ color: paint, specular: 0x262626, shininess: 34 }));
    m.insert(Slot::Trim, phong!({ color: 0x1a1b1d, specular: 0x1c1c1c, shininess: 40 }));
    m.insert(Slot::Metal, phong!({ color: 0x8c8e92, specular: 0x7a7a7a, shininess: 60 }));
    m.insert(Slot::Coil, phong!({ color: dim(0x9fa2a6), specular: 0x8a8a8a, shininess: 70 }));
    m.insert(Slot::Tray, phong!({ color: dim(0x6e7074), specular: 0x404040, shininess: 30 }));
    let liner = phong!({ color: dim(0xd2d0c8), specular: 0x202020, shininess: 20 });
    if powered {
        liner.set_emissive(0x3a3e44);
    }
    m.insert(Slot::Liner, liner);
    m.insert(Slot::Goods, phong!({ map: goods_atlas(), color: dim(0xffffff), specular: 0x343434, shininess: 42 }));
    m.insert(Slot::Bottle, phong!({ color: dim(0x8fb2c2), specular: 0x5a5a5a, shininess: 80 }));
    m.insert(Slot::Labels, lambert!({ map: strips.clone(), color: dim(0xffffff) }));
    let tube = if powered { basic_mat!({ color: 0xf2f6ff }) } else { phong!({ color: 0x9a9a94, specular: 0x404040, shininess: 60 }) };
    m.insert(Slot::Tube, tube);
    m.insert(Slot::Glass, phong!({ map: glass_tex(), color: 0xc8d4d4, transparent: true, opacity: 1.0, depthWrite: false, specular: 0x505050, shininess: 90 }));
    m.insert(Slot::Flap, phong!({ color: 0x1a1c1e, transparent: true, opacity: 0.82, specular: 0x404040, shininess: 70 }));
    m.insert(Slot::Decal, lambert!({ map: decal_tex(), transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 }));
    m.insert(Slot::Keys, phong!({ color: 0xb4b4ae, specular: 0x303030, shininess: 50 }));
    m.insert(Slot::Legend, lambert!({ map: keys_tex(), transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 }));
    m
}

/// A vending machine, for `scatter` to stand against a wall (its face out
/// from the wall): on (60%) or dead, painted one of five colours, stocked.
/// Its state is pushed to `Game::vending`; `placed` finishes it where it stands.
pub fn build_vending(g: &mut Game, cx: &mut PropsCtx) -> Built {
    let _ = cx;
    // one draw from the build's stream, the rest from a stream of the
    // machine's own (`with_own_stream`, not counted by the build's trace):
    // what it is made of never moves what is placed after it (the wall
    // `scatter` stands it against is drawn next)
    let seed = (rnd() * 4_294_967_296.0) as u32 ^ VEND_SALT;
    crate::base::rng::with_own_stream(seed, || build_machine(g, seed))
}

/// The salt of a machine's own stream
const VEND_SALT: u32 = 0x5EED_7E2D;

fn build_machine(g: &mut Game, seed: u32) -> Built {
    let powered = rnd() < 0.6;
    let paint = choice(&PAINTS);
    let places = coil_places();
    let stock = stock(&places, powered);
    let mut pieces = hardware(&places);
    let mut next = 0;
    let items: Vec<Vec<usize>> =
        places.iter().zip(&stock).enumerate().map(|(k, (c, s))| goods_pieces(k, c, s, &mut next, &mut pieces)).collect();
    let (strips, labels) = strip_pieces(&places);
    pieces.extend(labels);
    pieces.extend(decal_pieces());
    // the glass, set back in the door
    let glass = plane_geometry(WIN[1] - WIN[0], WIN[3] - WIN[2], 1.0, 1.0);
    pieces.push(Piece {
        geo: glass,
        m: at((WIN[0] + WIN[1]) / 2.0, (WIN[2] + WIN[3]) / 2.0, GLASS_Z),
        slot: Slot::Glass,
        coil: None,
        cell: None,
        item: None,
    });
    // age: rust and scuffs along the door's bottom (more on a dead one)
    let rust = plane_geometry(W - 0.02, 0.16, 1.0, 1.0);
    pieces.push(Piece { geo: rust, m: at(0.0, KICK + 0.08, FACE + 0.0012), slot: Slot::Rust, coil: None, cell: None, item: None });
    // the power cord, out of its back low down and along the floor to the wall
    let cord = cord_geo();
    pieces.push(Piece { geo: cord, m: Mat4::IDENTITY, slot: Slot::Trim, coil: None, cell: None, item: None });
    // the machine stands on its feet (`root`, placed by `scatter`); it rocks
    // and falls about its front bottom edge (`tilt`); its body hangs from that
    let root = group();
    let tilt = group();
    tilt.set_pos(0.0, 0.0, FACE);
    // the lock-side front leg is set half a turn lower, as the makers
    // say: it leans a hair forward and to that side
    tilt.rotation().x = 0.003;
    tilt.rotation().z = -0.004;
    let inner = group();
    inner.set_pos(0.0, 0.0, -FACE);
    tilt.add(&inner);
    root.add(&tilt);
    let mut mats = materials(paint, powered, &strips);
    mats.insert(Slot::Rust, lambert!({ map: rust_tex(), transparent: true, opacity: if powered { 0.55 } else { 1.0 }, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 }));
    let mut body = Body { root: inner.clone(), pieces, mats, meshes: Default::default(), lifted: None };
    body.draw(None);
    // a dead one now and then has a sign taped over its glass
    if !powered && rnd() < 0.5 {
        let sign = handwritten_tex(
            "OUT OF ORDER",
            &HandOpts {
                color: "rgba(20,15,10,0.95)".into(),
                thickness: 6.0,
                jitter: 1.8,
                wobble: 1.4,
                w: 512.0,
                h: 128.0,
                ..Default::default()
            },
        );
        let paper = lambert!({ color: 0xd8d2c2 });
        let ink = lambert!({ map: sign, transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 });
        let note = group();
        note.set_pos((WIN[0] + WIN[1]) / 2.0 + rand2(-0.08, 0.08), rand2(1.05, 1.35), GLASS_Z + 0.004);
        note.rotation().z = rand2(-0.12, 0.12);
        note.add(&mesh(plane_geometry(0.30, 0.13, 1.0, 1.0), &paper));
        let words = mesh(plane_geometry(0.29, 0.10, 1.0, 1.0), &ink);
        words.set_pos(0.0, 0.0, 0.0008);
        note.add(&words);
        let tape = lambert!({ color: 0xcfc4a0, transparent: true, opacity: 0.8 });
        for (x, y) in [(-0.13, 0.055), (0.13, 0.055)] {
            let t = mesh(plane_geometry(0.06, 0.02, 1.0, 1.0), &tape);
            t.set_pos(x, y, 0.0012);
            t.rotation().z = rand2(-0.5, 0.5);
            note.add(&t);
        }
        inner.add(&note);
    }
    // the flap, its own mesh hinged at its top
    let (fx, fy, fz) = flap_pivot();
    let hinge = group();
    hinge.set_pos(fx, fy, fz);
    let flap = mesh(part("Vending__flap"), &body.mats[&Slot::Flap]);
    flap.set_pos(0.0, -(BIN[3] - BIN[2] - 0.012) / 2.0 + 0.004, 0.0);
    flap.set_render_order(2);
    hinge.add(&flap);
    inner.add(&hinge);
    // the display
    let (dx, dy, dw, dh) = display_rect();
    let mut canvas = Canvas::new(DISPLAY_W as u32, DISPLAY_H as u32);
    paint_display(&mut canvas, "", 0, 0.0);
    let display_tex = canvas.to_tex();
    {
        let mut t = display_tex.t();
        t.color_space = ColorSpace::Srgb;
    }
    let display = mesh(plane_geometry(dw, dh, 1.0, 1.0), basic_mat!({ map: display_tex.clone(), color: 0xffffff }));
    display.set_pos(dx, dy, PANEL_FACE - 0.004);
    inner.add(&display);
    // its light: the tube's on the goods (a soft spot down into the
    // cabinet, so nothing outside it is lit through its walls), and the
    // glow out of the glass onto the floor and the walls by it
    let mut lights = vec![];
    if powered {
        let (tx, ty, tz) = tube_xyz();
        let l = spot_light!(0xe8eeff, 2.4, 1.6, 1.1, 0.8, 1.2);
        l.set_pos(tx, ty - 0.03, tz - 0.02);
        let aim = group();
        aim.set_pos(tx, 0.45, tz - 0.30);
        inner.add(&aim);
        l.light().target = Some(aim.clone());
        inner.add(&l);
        lights.push(l);
        let glow = point_light!(0xdfe8ff, 1.1, 3.4, 2.0);
        glow.set_pos(CAB[0], 1.15, FACE + 0.35);
        inner.add(&glow);
        lights.push(glow);
    }
    let tubes = body.mats[&Slot::Tube].clone();
    let light_power: Vec<f64> = lights.iter().map(|l| l.intensity()).collect();
    // a dead one: a hand print inside its glass, for later (rule 9); and
    // now and then a good already hanging (someone else's coin)
    let mut print = None;
    let mut hanging = None;
    if !powered {
        let h = mesh(
            plane_geometry(0.16, 0.22, 1.0, 1.0),
            lambert!({ map: handprint_tex(), transparent: true, opacity: 0.0, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -1, polygonOffsetUnits: -1 }),
        );
        h.set_pos((WIN[0] + WIN[1]) / 2.0 + rand2(-0.15, 0.15), rand2(1.05, 1.35), GLASS_Z - 0.003);
        h.rotation().z = rand2(-0.3, 0.3);
        h.set_visible(false);
        h.set_render_order(3);
        inner.add(&h);
        print = Some(h);
        if rnd() < 0.5 {
            let k = (0..places.len()).filter(|&k| stock[k].count > 0).nth(irand(0, 9) as usize);
            if let Some(k) = k {
                hang_front(&mut body, &places[k], items[k][0]);
                body.draw(None);
                hanging = Some(k);
            }
        }
    }
    let full: Vec<usize> = stock.iter().map(|s| s.count).collect();
    g.vending.push(Machine {
        root: root.clone(),
        tilt,
        body,
        places,
        stock,
        powered,
        display,
        display_canvas: canvas,
        display_tex,
        items,
        flap: hinge,
        tubes,
        lights,
        light_power,
        inside: vec![],
        print,
        seed,
        full,
        hum: None,
        // how many shoves shake what hangs loose: its own, 2 to 4 (rule 6)
        state: rules::State {
            drop_on: irand(2, 4) as u32,
            // rule 1: on 40-60 s, dark 6-9 s, from a point in it of its own
            rhythm: (rand2(40.0, 60.0), rand2(6.0, 9.0), rand1(70.0)),
            lit: powered,
            hanging,
            // rule 9: half the dead ones have something in them (a bit of
            // its seed, not a draw: nothing after it moves)
            inhabited: !powered && (seed >> 9) & 1 == 1,
            ..Default::default()
        },
    });
    Built::new(root, H + 0.01)
}

/// `scatter` has stood the machine just built: its uses (where it finally
/// stands is read from its node, after the level's settling)
pub fn placed(g: &mut Game, built: &Built, si: &crate::core_props::SpotInfo) {
    let _ = si;
    let Some(i) = g.vending.iter().rposition(|m| m.root.ptr_eq(&built.g)) else { return };
    g.vending_register(i);
}
