//! The Jahamun Tunnel set: the approach to the portal at night, the long
//! stairs down the retaining wall, and the 485 m walkway inside the bore,
//! behind the glass noise barrier (`crates/jahamun/README.md`).
//!
//! The set is the game's kind of thing and is built to move in: it reads
//! only the engine, draws no random number, reads no clock, and keeps its
//! light in its places (the bake's, and fog chosen by where each surface
//! is, never by where you are). What you get:
//!
//! * [`Set::build`]: the scene graph (`root`), the walkable ground
//!   ([`walk::Walk`]), the spawns;
//! * [`Set::update`]: once a frame with the eye's position: the highlights
//!   and the glass's reflection fade with distance and angle, the glows
//!   keep their size in the eye, the other bore is let go when it cannot be
//!   seen;
//! * [`Set::globals`]: the render globals (night, the fog by place).
//!
//! It is made by `pipeline/` (textures from CC0 photographs, the place
//! built and its light baked in Blender) into `assets/`.
pub mod assets;
pub mod data;
pub mod look;
pub mod ring;
pub mod signs;
pub mod walk;

use data::{MatKind, Piece, Placement, SetData};
use engine::dmath::DMath;
use engine::prelude::*;
use engine::render::{LocalLight, RenderGlobals};
use look::{Looks, Textures};
use std::collections::BTreeMap;

/// The glass's plane (the engine's x), mirrored about by its reflection.
const GLASS_X: f64 = 0.10;
/// What the walkway's glass is seen to reflect: the lights' faces below the
/// head rail (a reflection without its surfaces' pictures would read as
/// ghosts; the lights doubled in the glass are what the video shows).
const REFLECTED: &[&str] = &["led"];
const REFLECT_TOP: f32 = 3.15;

/// How far from the eye a module's highlights are drawn (m).
const HIGHLIGHT_REACH: f64 = 60.0;

/// The night outside (as displayed, sRGB) and the haze in the bores.
const NIGHT_FOG: [f64; 3] = [0.075, 0.066, 0.064];
const NIGHT_DENSITY: f64 = 0.010;
const HAZE: [f64; 3] = [0.50, 0.535, 0.52];
const HAZE_RATIO: f64 = 0.42;
/// The haze in the bores of the way in, whose walkway has no end: thick
/// enough that at the ring's reach (`ring::VIEW` modules, 252 m) 1% of what
/// is there is left, so neither the arch behind you nor the end of what is
/// built ahead can be seen.
const HAZE_RATIO_RING: f64 = 0.85;
const SKY_TOP: [f64; 3] = [0.004, 0.0045, 0.007];
/// Seoul's night: dark overhead, an orange-grey glow low down
const SKY_LOW: [f64; 3] = [0.070, 0.048, 0.033];

/// A placed copy whose look changes with the eye: a module of the
/// walkway's bore (its highlights, its glass, its reflection).
struct Live {
    centre: Vec3,
    highlights: Vec<(Obj, Mat, f64)>,
    glass: Option<Mat>,
    reflection: Option<(Obj, Mat)>,
}

struct Glow {
    node: Obj,
    at: Vec3,
    size: f64,
}

/// How the walkway is laid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layout {
    /// The real tunnel: 485 m from the portal to the far gate.
    Real,
    /// The way in: the walkway as a ring with no end (`ring`), the escape
    /// door under its plate once a period.
    Ring,
}

/// The escape door where it comes round (the ring's F modules): its place
/// along the bore and its two looks, shut and changed.
pub struct DoorSlot {
    /// the door's middle along the bore (z)
    pub z: f64,
    shut: Obj,
    changed: Obj,
}

/// The escape door's middle across (x) and the module's z of it.
pub const DOOR_X: f64 = -1.70;
const DOOR_Z_IN_MODULE: f64 = -3.0;
/// Her note beside the changed door: its middle (along the bore, from the
/// door's middle) and its height.
pub const NOTE_FROM_DOOR: f64 = 0.80;
pub const NOTE_Y: f64 = 1.47;

pub struct Set {
    pub root: Obj,
    pub walk: walk::Walk,
    /// name -> x, y (feet), z, heading
    pub spawns: BTreeMap<String, [f64; 4]>,
    pub tunnel: f64,
    live: Vec<Live>,
    glows: Vec<Glow>,
    bore2: Obj,
    fog_map: Tex,
    fog_grid: [f64; 4],
    haze_ratio: f64,
    pub layout: Layout,
    /// the ring's state (the way in only)
    pub ring: ring::Ring,
    doors: Vec<DoorSlot>,
    door_changed: bool,
    /// her note's sheets (by each changed door)
    notes: Vec<Obj>,
}

/// A piece's meshes made into geometry once (shared by its placements):
/// (material, geometry); and its glass reflection, if it has one.
struct Built {
    meshes: Vec<(String, Geo)>,
    reflection: Option<Geo>,
}

fn v3(a: [f64; 3]) -> Vec3 {
    Vec3::new(a[0], a[1], a[2])
}

fn build_piece(d: &SetData, name: &str, p: &Piece, looks: &Looks) -> Built {
    let mut meshes = vec![];
    let (mut rp, mut rc, mut ri): (Vec<f32>, Vec<f32>, Vec<u32>) = (vec![], vec![], vec![]);
    let tunnel_piece = name.starts_with("module_") || name == "far" || name == "entrance";
    for m in &p.meshes {
        let (v0, nv) = m.vertices;
        let (i0, ni) = m.indices;
        let def = &d.materials[&m.material];
        let geo = Geo::new();
        geo.set_position(d.position[v0 * 3..(v0 + nv) * 3].to_vec());
        geo.set_normal(d.normal[v0 * 3..(v0 + nv) * 3].to_vec());
        geo.set_uv(d.uv[v0 * 2..(v0 + nv) * 2].to_vec());
        let idx: Vec<u32> = d.index[i0..i0 + ni].to_vec(); // (each mesh's own: from 0)
        let mean = looks.means.get(&m.material).copied().unwrap_or([0.5; 3]);
        let mut col = Vec::with_capacity(nv * 3);
        let lit = matches!(def.kind, MatKind::Baked | MatKind::Sign);
        for k in 0..nv {
            let l = [d.light[(v0 + k) * 3], d.light[(v0 + k) * 3 + 1], d.light[(v0 + k) * 3 + 2]];
            let c = if lit {
                let p = [d.position[(v0 + k) * 3], d.position[(v0 + k) * 3 + 1], d.position[(v0 + k) * 3 + 2]];
                let g = look::grime(&m.material, p) as f32;
                let t = look::tone(l, mean);
                [t[0] * g, t[1] * g, t[2] * g]
            } else if def.kind == MatKind::Glass {
                // a pane in the bore shows the lit tunnel; outside, the night
                let z = d.position[(v0 + k) * 3 + 2];
                let inside = name != "entrance" || z < -0.5;
                if inside { [1.0, 1.0, 1.0] } else { [0.08, 0.085, 0.1] }
            } else {
                [1.0, 1.0, 1.0]
            };
            col.extend_from_slice(&c);
        }
        // the reflection: the walkway's side below the head rail, mirrored in the glass
        if tunnel_piece && REFLECTED.contains(&m.material.as_str()) {
            let face = def.kind == MatKind::Emissive;
            for t in idx.as_chunks::<3>().0 {
                let ok = t.iter().all(|&i| {
                    let i = i as usize;
                    let (x, y, z) = (d.position[(v0 + i) * 3], d.position[(v0 + i) * 3 + 1], d.position[(v0 + i) * 3 + 2]);
                    (x as f64) < GLASS_X - 0.01 && y < REFLECT_TOP && (name != "entrance" || z < 0.0)
                });
                if !ok {
                    continue;
                }
                let base = (rp.len() / 3) as u32;
                for &i in t.iter().rev() {
                    let i = i as usize;
                    let (x, y, z) = (d.position[(v0 + i) * 3], d.position[(v0 + i) * 3 + 1], d.position[(v0 + i) * 3 + 2]);
                    rp.extend_from_slice(&[(2.0 * GLASS_X) as f32 - x, y, z]);
                    if face {
                        let f = look::face(def.tint);
                        rc.extend_from_slice(&[f.r as f32 * 1.6, f.g as f32 * 1.6, f.b as f32 * 1.6]);
                    } else {
                        let c = &col[i * 3..i * 3 + 3];
                        rc.extend_from_slice(&[c[0] * mean[0] as f32, c[1] * mean[1] as f32, c[2] * mean[2] as f32]);
                    }
                }
                ri.extend_from_slice(&[base, base + 1, base + 2]);
            }
        }
        if lit || def.kind == MatKind::Glass {
            geo.set_color(col, 3);
        }
        geo.set_index(idx);
        geo.compute_bounding_sphere();
        meshes.push((m.material.clone(), geo));
    }
    let reflection = (!ri.is_empty()).then(|| {
        let g = Geo::new();
        g.set_position(rp);
        g.set_color(rc, 3);
        g.set_index(ri);
        g.compute_bounding_sphere();
        g
    });
    Built { meshes, reflection }
}

/// The night sky: a dome, dark overhead, the city's glow low down.
fn sky() -> Obj {
    let g = sphere_geo!(700, 32, 16);
    {
        let mut gg = g.g();
        let n = gg.position.len() / 3;
        let mut col = Vec::with_capacity(n * 3);
        for k in 0..n {
            let y = gg.position[k * 3 + 1] as f64 / 700.0;
            let t = (y.max(0.0)).dpowf(0.45);
            for c in 0..3 {
                col.push((SKY_LOW[c] + (SKY_TOP[c] - SKY_LOW[c]) * t) as f32);
            }
        }
        gg.color = col;
        gg.color_size = 3;
        gg.version += 1;
    }
    let m = basic_mat!({ vertexColors: true, side: BackSide, depthWrite: false, fog: false });
    let o = mesh(g, m);
    o.set_render_order(-10);
    o.set_frustum_culled(false);
    o
}

impl Set {
    /// The set, from what is embedded (`assets`), laid out as `layout`.
    pub fn build(layout: Layout) -> Result<Set, String> {
        let mut d = SetData::read(assets::SET)?;
        if layout == Layout::Ring {
            lay_ring(&mut d);
        }
        let mut tex = Textures::new();
        let looks = Looks::new(&d.materials, &mut tex);
        let root = group();
        root.set_name("jahamun");
        root.add(&sky());
        let built: BTreeMap<String, Built> = d.pieces.iter().map(|(n, p)| (n.clone(), build_piece(&d, n, p, &looks))).collect();

        // the signs' pictures
        let exit = signs::exit_sign();
        let name_plate = signs::name_plate();
        let gate = signs::gate_notice();
        let arrow = signs::lane_arrow();
        let sign_mat = |base: &Mat, t: &Tex| {
            let m = base.duplicate();
            m.m().map = Some(t.clone());
            m
        };
        let exit_mat = sign_mat(&looks.mats["sign_exit"], &exit);
        let name_mat = sign_mat(&looks.mats["sign_name"], &name_plate);
        // the way-out sign is lit from within, the name's letters by a lamp over them
        exit_mat.m().color = Color::rgb_linear(2.4, 2.4, 2.4);
        name_mat.m().color = Color::rgb_linear(1.8, 1.8, 1.8);
        let gate_mat = sign_mat(&looks.mats["sign_gate"], &gate);
        let arrow_mat = sign_mat(&looks.mats["signal_green"], &arrow);
        arrow_mat.m().color = Color::rgb_linear(1.0, 1.0, 1.0);
        let note_tex = signs::her_note();

        let floor_gloss = tex.gloss("floor_gloss.jpg");
        let tile_gloss = tex.gloss("tile_gloss.jpg");
        let glow_tex = look::glow_texture();
        let glow_mat = sprite_mat!({ map: &glow_tex, blending: AdditiveBlending, depthWrite: false, transparent: true });
        let glow_red = glow_mat.duplicate();
        let glow_green = glow_mat.duplicate();
        let glow_warm = glow_mat.duplicate();
        glow_red.m().color = Color::rgb_linear(1.0, 0.12, 0.08);
        glow_green.m().color = Color::rgb_linear(0.3, 1.0, 0.5);
        glow_warm.m().color = Color::rgb_linear(1.0, 0.84, 0.66);
        glow_mat.m().color = Color::rgb_linear(0.82, 0.88, 0.9);

        let bore2 = group();
        bore2.set_name("the other bore");
        root.add(&bore2);
        let lights = group();
        lights.set_name("highlight lights");
        root.add(&lights);
        let mut live = vec![];
        let mut glows = vec![];
        let mut escape_no = 0;
        let mut doors: Vec<DoorSlot> = vec![];
        let mut notes: Vec<Obj> = vec![];
        for pl in &d.placements {
            let Some(b) = built.get(&pl.piece) else { continue };
            let piece = &d.pieces[&pl.piece];
            let node = group();
            node.set_name(&pl.piece);
            node.set_pos(pl.at[0], pl.at[1], pl.at[2]);
            let mirrored = pl.mirror.is_some();
            if let Some(m) = pl.mirror {
                node.set_pos(m, pl.at[1], pl.at[2]);
                node.set_scale(-1.0, 1.0, 1.0);
            }
            let second = pl.bore == 2 || pl.piece == "entrance2";
            let k = pl.index.unwrap_or(0);
            let mut inst =
                Live { centre: Vec3::new(-0.8, 1.5, pl.at[2] - d.module / 2.0), highlights: vec![], glass: None, reflection: None };
            for (mat_name, geo) in &b.meshes {
                let base = if mirrored { &looks.mats_mirrored[mat_name] } else { &looks.mats[mat_name] };
                let mat = match mat_name.as_str() {
                    "sign_exit" => exit_mat.clone(),
                    "sign_name" => name_mat.clone(),
                    "sign_gate" => gate_mat.clone(),
                    "signal_green" => arrow_mat.clone(),
                    "sign_info" => {
                        escape_no += 1;
                        let (n, back, ahead) = match layout {
                            Layout::Real => {
                                let at = -pl.at[2] + 2.6;
                                (escape_no, at.round() as i64, (d.tunnel - at).round() as i64)
                            }
                            // the ring's plates say what they would on its first lap, every lap
                            Layout::Ring => ring_plate(&pl.piece, k as usize, d.tunnel),
                        };
                        sign_mat(base, &signs::info_plate(n, back, ahead))
                    }
                    "note" => sign_mat(base, &note_tex),
                    "glass" if !second => {
                        let g = base.duplicate();
                        if pl.index.is_some() {
                            inst.glass = Some(g.clone());
                        }
                        g
                    }
                    _ => base.clone(),
                };
                let m = mesh(geo.clone(), mat);
                if mat_name == "note" {
                    m.set_name("her note");
                    notes.push(m.clone());
                }
                node.add(&m);
                // the highlights on the walkway's wet paint and glazed tiles (the walkway's bore
                // only): every light within reach of a surface is looped over at each of its
                // pixels, so only the bars light them (two points a bar, 4 m), and the street lamps
                // outside
                if !second {
                    let hl = match mat_name.as_str() {
                        "floor" => Some((&floor_gloss, 0.24, 110.0, 1.0)),
                        "plaza" => Some((&floor_gloss, 0.16, 90.0, 1.0)),
                        "tile" | "tile_green" => Some((&tile_gloss, 0.10, 180.0, 0.85)),
                        _ => None,
                    };
                    if let Some((g, s, n, o)) = hl {
                        let hm = look::highlight(g, s, n);
                        hm.m().opacity = o;
                        let ho = mesh(geo.clone(), hm.clone());
                        node.add(&ho);
                        inst.highlights.push((ho, hm, o));
                    }
                }
            }
            if !second && let Some(r) = &b.reflection {
                let rm = basic_mat!({ vertexColors: true, transparent: true, blending: AdditiveBlending, depthWrite: false, fog: false, opacity: 0.1 });
                let ro = mesh(r.clone(), rm.clone());
                node.add(&ro);
                inst.reflection = Some((ro, rm));
            }
            // the lights: for the highlights (the walkway's bore and outside) and the glows
            for l in &piece.lights {
                let mut at = v3(l.at);
                if let Some(m) = pl.mirror {
                    at.x = m - at.x;
                }
                at.x += if mirrored { 0.0 } else { pl.at[0] };
                at.y += pl.at[1];
                at.z += pl.at[2];
                let col = Color::rgb_linear(l.colour[0], l.colour[1], l.colour[2]);
                let (reach, power, glow, gsize) = match l.kind.as_str() {
                    "led" => (4.0, 6.0, Some(&glow_mat), 0.17),
                    "lamp" => (0.0, 0.0, Some(&glow_mat), 0.9),
                    "street" => (30.0, 110.0, Some(&glow_warm), 2.4),
                    // (the strip under the balustrade's rail: its glow only; its 54 points
                    // looped over at every pixel of the plaza cost more than its glints)
                    "rail" => (0.0, 0.0, Some(&glow_mat), 0.12),
                    "red" => (3.0, 1.5, Some(&glow_red), 0.35),
                    "signal" => (0.0, 0.0, Some(&glow_green), 0.9),
                    _ => (0.0, 0.0, None, 0.0),
                };
                if power > 0.0 && !second {
                    let along = v3(l.axis);
                    let n = if l.kind == "led" { 2 } else { 1 };
                    for s in 0..n {
                        let off = if n == 1 { 0.0 } else { (s as f64 / (n - 1) as f64 - 0.5) * l.size[1] * 0.8 };
                        let p = point_light!(col, power / n as f64, reach, 2.0);
                        p.set_pos(at.x + along.x * off, at.y + along.y * off, at.z + along.z * off);
                        lights.add(&p);
                    }
                }
                if let Some(gm) = glow {
                    // a bar's glow is two along it (a line, not a blob)
                    let along = v3(l.axis);
                    let offs: &[f64] = if l.kind == "led" { &[-0.33, 0.33] } else { &[0.0] };
                    for &o in offs {
                        let s = sprite(gm);
                        let dir = v3(l.dir);
                        let p = Vec3::new(
                            at.x + dir.x * 0.04 + along.x * o,
                            at.y + dir.y * 0.04 + along.y * o,
                            at.z + dir.z * 0.04 + along.z * o,
                        );
                        s.set_pos(p.x, p.y, p.z);
                        s.set_scale(gsize, gsize, gsize);
                        if second {
                            bore2.add(&s)
                        } else {
                            root.add(&s)
                        };
                        glows.push(Glow { node: s, at: p, size: gsize });
                    }
                }
            }
            if second && pl.piece != "entrance2" {
                bore2.add(&node);
            } else {
                root.add(&node);
            }
            // the escape door's two looks, where it comes round (the ring)
            match pl.piece.as_str() {
                "module_F" => doors.push(DoorSlot { z: pl.at[2] + DOOR_Z_IN_MODULE, shut: node.clone(), changed: node.clone() }),
                "module_G" => {
                    if let Some(slot) = doors.iter_mut().find(|s| (s.z - (pl.at[2] + DOOR_Z_IN_MODULE)).abs() < 1e-6) {
                        slot.changed = node.clone();
                    }
                    node.set_visible(false);
                }
                _ => {}
            }
            if pl.index.is_some() && !second {
                let _ = k;
                live.push(inst);
            }
        }

        // the fog by place: a light map over the set whose red is how lit each place is
        let deepest = d.lit.iter().map(|b| -b.z[0]).fold(d.tunnel, f64::max);
        let (x0, x1, z0, z1) = (-40.0, 40.0, -(deepest + 40.0), 100.0);
        let (w, h) = (64usize, 640usize);
        let mut px = vec![0u8; w * h];
        for j in 0..h {
            for i in 0..w {
                let x = x0 + (i as f64 + 0.5) / w as f64 * (x1 - x0);
                let z = z0 + (j as f64 + 0.5) / h as f64 * (z1 - z0);
                let mut lit: f64 = 0.0;
                for b in &d.lit {
                    // inside the bore lit; out of its portal the light dies over 10 m
                    let dx = (b.x[0] - x).max(x - b.x[1]).max(0.0);
                    let dz = (b.z[0] - z).max(z - b.z[1]).max(0.0);
                    let out = (dx * dx + dz * dz).sqrt();
                    lit = lit.max(b.lit * (1.0 - out / 10.0).clamp(0.0, 1.0).dpowi(2));
                }
                px[j * w + i] = (lit * 255.0 + 0.5) as u8;
            }
        }
        let fog_map = Tex::data_r8(px, w as u32, h as u32);
        {
            let mut t = fog_map.t();
            t.flip_y = false;
            t.generate_mipmaps = false;
            t.min_filter = Filter::Linear;
            t.mag_filter = Filter::Linear;
        }
        let fog_grid = [1.0 / (x1 - x0), 1.0 / (z1 - z0), -x0 / (x1 - x0), -z0 / (z1 - z0)];

        Ok(Set {
            root,
            walk: walk::Walk::new(d.walk.clone()),
            spawns: d.spawns.clone(),
            tunnel: d.tunnel,
            live,
            glows,
            bore2,
            fog_map,
            fog_grid,
            haze_ratio: if layout == Layout::Ring { HAZE_RATIO_RING } else { HAZE_RATIO },
            layout,
            ring: ring::Ring::default(),
            doors,
            door_changed: false,
            notes,
        })
    }

    /// Her note taken from beside the door (every copy): the tape stays.
    pub fn take_note(&mut self) {
        for n in &self.notes {
            n.set_visible(false);
        }
    }

    /// The escape door where it comes round (the ring's), each copy.
    pub fn doors(&self) -> impl Iterator<Item = f64> + '_ {
        self.doors.iter().map(|s| s.z)
    }

    /// Whether the escape door has changed.
    pub fn door_changed(&self) -> bool {
        self.door_changed
    }

    /// The escape door changed (or back), every copy at once.
    pub fn set_door_changed(&mut self, changed: bool) {
        self.door_changed = changed;
        for s in &self.doors {
            s.shut.set_visible(!changed);
            s.changed.set_visible(changed);
        }
    }

    /// The render globals: the night, and the fog by place (the bores'
    /// lit haze, the dark outside).
    pub fn globals(&self) -> RenderGlobals {
        RenderGlobals {
            background: Color::rgb_linear(SKY_TOP[0], SKY_TOP[1], SKY_TOP[2]),
            fog: Some((Color::rgb_linear(NIGHT_FOG[0], NIGHT_FOG[1], NIGHT_FOG[2]), NIGHT_DENSITY)),
            ll: LocalLight {
                on: true,
                map: Some(self.fog_map.clone()),
                grid: self.fog_grid,
                scale: 1.0,
                amb_d: Vec3::ZERO,
                sky_d: Vec3::ZERO,
                gnd_d: Vec3::ZERO,
                fog_lit: Color::rgb_linear(HAZE[0], HAZE[1], HAZE[2]),
                fog_ratio: self.haze_ratio,
            },
            pixel_ratio: 1.0,
            ..Default::default()
        }
    }

    /// Once a frame, with the eye's position: the highlights fade into the
    /// haze, the glass reflects more the flatter you see it, the glows keep
    /// a size in the eye, and the other bore is let go from deep inside.
    pub fn update(&mut self, eye: Vec3) {
        let on_walkway = eye.x < GLASS_X && eye.z < 1.0 && eye.y < 3.0;
        let density = NIGHT_DENSITY * self.haze_ratio;
        for l in &mut self.live {
            let d = l.centre.distance_to(&eye);
            let haze = (-(density * d).dpowi(2)).dexp();
            // (past `HIGHLIGHT_REACH` a glint off the paint or the glaze is a
            // pixel or two: its pass, the densest meshes drawn again, is let go)
            let near = d < HIGHLIGHT_REACH;
            for (o, m, base) in &l.highlights {
                o.set_visible(near);
                m.m().opacity = base * haze;
            }
            // Fresnel on the glass: square on 4% a face, all of it at a graze
            let dx = (GLASS_X - eye.x).abs().max(0.05);
            let cos = dx / ((dx * dx + (l.centre.z - eye.z).dpowi(2) + 0.6).sqrt());
            let fr = (0.08 + 0.92 * (1.0 - cos).dpowi(5)).min(0.85);
            if let Some(g) = &l.glass {
                g.m().opacity = (0.30 + 0.45 * fr).min(0.75);
            }
            if let Some((o, m)) = &l.reflection {
                o.set_visible(on_walkway);
                m.m().opacity = fr * haze;
            }
        }
        for g in &self.glows {
            let d = g.at.distance_to(&eye).max(0.5);
            // a glow keeps a little of its size in the eye as it recedes
            let s = g.size * (d / 4.0).max(1.0).dpowf(0.35);
            g.node.set_scale(s, s, s);
        }
        let deep = eye.x < 1.0 && eye.z < -15.0;
        self.bore2.set_visible(!deep);
    }

    /// A spawn: the eye's position (eye height over the feet) and heading.
    pub fn spawn(&self, name: &str) -> Option<([f64; 3], f64)> {
        self.spawns.get(name).map(|s| ([s[0], s[1], s[2]], s[3]))
    }
}

/// The way in's walkway (`Layout::Ring`): the entrance, then the ring's
/// modules (`ring`) as far as can be seen past both its seams, the escape
/// door's changed look beside each of its own, the other bore only as far
/// as it is seen from outside, no far end; the walkway's ground and the
/// bores' light as long as what is built.
fn lay_ring(d: &mut SetData) {
    let module = d.module;
    let mut place: Vec<Placement> = d.placements.iter().filter(|p| p.piece.starts_with("entrance")).cloned().collect();
    for k in ring::FIRST..ring::MODULES {
        let v = ring::variant(k);
        let at = [0.0, 0.0, -(k as f64) * module];
        place.push(Placement { piece: format!("module_{v}"), at, index: Some(k as i64), mirror: None, bore: 1 });
        if v == "F" {
            place.push(Placement { piece: "module_G".into(), at, index: Some(k as i64), mirror: None, bore: 1 });
        }
        if k < 12 {
            place.push(Placement { piece: "module_A".into(), at, index: Some(k as i64), mirror: Some(d.mirror), bore: 2 });
        }
    }
    d.placements = place;
    let built = ring::MODULES as f64 * module;
    for w in d.walk.iter_mut().filter(|w| w.name == "walkway") {
        w.z[0] = -built + 0.5;
    }
    for b in d.lit.iter_mut().filter(|b| b.z[0] < -1.0) {
        b.z[0] = -built - module;
    }
}

/// What an escape plate on the ring says (its number, metres back to the
/// arch and on to the far portal): what it would on the ring's first lap,
/// every lap.
fn ring_plate(piece: &str, k: usize, tunnel: f64) -> (i64, i64, i64) {
    let first = ring::FIRST + (k - ring::FIRST) % ring::PERIOD.len();
    let (n, in_module) = match piece {
        "module_C" => (1, 4.6),
        _ => (2, 1.65),
    };
    let at = first as f64 * ring::MOD + in_module;
    (n, at.round() as i64, (tunnel - at).round() as i64)
}
