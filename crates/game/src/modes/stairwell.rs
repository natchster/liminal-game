//! The Stairwell (`modes::stairwell`): the unlit staircase between some levels.
//!
//! Inspired by SCP-087 ("SCP-087" by Zaeyde, CC BY-SA 3.0,
//! https://scp-wiki.wikidot.com/scp-087). Our own names and text: this is
//! "the Stairwell" and "the Face". Flights of thirteen concrete steps at 38°,
//! a semicircular landing and a half turn, over and over; a light that never
//! reaches further than about nine steps; a child crying far below; and a
//! pale face that waits at the edge of the light, stares, and jerks nearer.
//! It never chases. But if you turn round and start back up, it is behind you.
//!
//! Geometry (plan view, metres, local to `group`): flights run along z
//! between z = 0 and z = FL; even flights on the left (x < 0) go down towards
//! +z, odd flights on the right go down towards -z; a half-disc landing
//! closes each end. Everything repeats every two flights (DROP·2 metres), so
//! the visible shaft is four recycled chunks.
//!
//! Movement state is a segment index: seg = 2k is flight k, seg = 2k+1 is
//! the landing after flight k (seg -1 is the landing at the top door).
//!
//! `Stairwell::new(StairDeps)` builds it; the pub fields and methods below
//! are what the game glue (`modes/mod.rs`) uses.

use crate::base::numeric::*;
use engine::prelude::*;
use std::cell::RefCell;
use std::f64::consts::PI;
use std::rc::Rc;

// ------------------------------------------------------------------ deps ---
/// A canvas texture maker `(w, h, repeat, draw)`: makes a canvas texture by
/// running `draw(canvas, w, h)` once. The game's is [`game_canvas_tex`].
pub type CanvasTexFn = Rc<dyn Fn(f64, f64, bool, &mut dyn FnMut(&mut Canvas, f64, f64)) -> Tex>;

/// The game's canvas textures (`placers::kit::canvas_tex_opt`) as a [`CanvasTexFn`].
pub fn game_canvas_tex() -> CanvasTexFn {
    Rc::new(|w: f64, h: f64, repeat: bool, draw: &mut dyn FnMut(&mut Canvas, f64, f64)| {
        crate::placers::kit::canvas_tex_opt(w, h, repeat, |c, w, h| draw(c, w, h))
    })
}

/// A random source shared by the module and its canvas draws (`StairDeps::rng`).
#[derive(Clone)]
pub struct SharedRng(pub Rc<RefCell<Box<dyn FnMut() -> f64>>>);
impl SharedRng {
    pub fn new(f: Box<dyn FnMut() -> f64>) -> SharedRng {
        SharedRng(Rc::new(RefCell::new(f)))
    }
    /// the next draw in [0, 1)
    #[inline]
    pub fn next(&self) -> f64 {
        (self.0.borrow_mut())()
    }
    /// a draw in [a, b): `a + next() * (b - a)`
    #[inline]
    pub fn range(&self, a: f64, b: f64) -> f64 {
        a + self.next() * (b - a)
    }
}

/// What the Stairwell is built with.
pub struct StairDeps {
    /// the canvas texture maker
    pub canvas_tex: CanvasTexFn,
    /// the random source (the game passes `base::rng::native_rnd`)
    pub rng: Box<dyn FnMut() -> f64>,
    /// the Face's geometry (the Blender face shell, painted); None for the procedural face
    pub face_geometry: Option<Geo>,
}
impl StairDeps {
    /// The game's deps: its canvas textures, the native RNG and the given face geometry.
    pub fn game(face_geometry: Option<Geo>) -> StairDeps {
        StairDeps { canvas_tex: game_canvas_tex(), rng: Box::new(crate::base::rng::native_rnd), face_geometry }
    }
}
impl Default for StairDeps {
    /// the game's canvas textures, the seeded `rnd()` and the procedural face
    fn default() -> Self {
        StairDeps { canvas_tex: game_canvas_tex(), rng: Box::new(crate::base::rng::rnd), face_geometry: None }
    }
}

// ------------------------------------------------------------- constants ---
#[allow(non_camel_case_types)]
pub struct STAIR;
impl STAIR {
    pub const STEPS: i32 = 13;
    pub const RISE: f64 = 0.24;
    /// 38° pitch: 0.24 / 0.31 = tan 37.7°
    pub const RUN: f64 = 0.31;
    /// shaft half-width
    pub const HALF: f64 = 1.45;
    /// spine half-width
    pub const SPINE: f64 = 0.1;
    /// landing radius
    pub const R: f64 = 1.45;
    /// the lamp never shows more than this
    pub const LIGHT_STEPS: i32 = 9;
}
/// 4.03 m of plan per flight
pub const STAIR_FL: f64 = STAIR::STEPS as f64 * STAIR::RUN;
/// 3.12 m of height per flight (`STAIR_FLIGHT_DROP`)
pub const STAIR_FLIGHT_DROP: f64 = STAIR::STEPS as f64 * STAIR::RISE;
const FL: f64 = STAIR_FL;
const DROP: f64 = STAIR_FLIGHT_DROP;
const HALF: f64 = STAIR::HALF;
const SPINE: f64 = STAIR::SPINE;
const R: f64 = STAIR::R;
const RUN: f64 = STAIR::RUN;
const RISE: f64 = STAIR::RISE;
const STEPS: i32 = STAIR::STEPS;

// ----------------------------------------------------------------- types ---
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FaceKind {
    Below,
    Behind,
}
impl FaceKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            FaceKind::Below => "below",
            FaceKind::Behind => "behind",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GoneReason {
    Stared,
    Approached,
    Timeout,
    Behind,
}
impl GoneReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            GoneReason::Stared => "stared",
            GoneReason::Approached => "approached",
            GoneReason::Timeout => "timeout",
            GoneReason::Behind => "behind",
        }
    }
}

/// Events `update()` returns (`StairOut::events`).
#[derive(Clone, Debug, PartialEq)]
pub enum StairEvent {
    /// `faceAppear`: an encounter below begins
    FaceAppear,
    /// `faceSeen`: the player first sees the Face, `dist` away
    FaceSeen { dist: f64 },
    /// `faceJerk`: the Face jerks nearer (the `n`th time), ending `dist` away
    FaceJerk { n: i32, dist: f64 },
    /// `faceGone`: the Face vanishes, for `reason`
    FaceGone { reason: GoneReason },
    /// `faceBehindArmed`: the Face is waiting behind the climbing player
    FaceBehindArmed,
    /// `faceBehind`: the player turned round onto it
    FaceBehind,
}
impl StairEvent {
    /// the event's name (`faceAppear`, ...)
    pub fn type_name(&self) -> &'static str {
        match self {
            StairEvent::FaceAppear => "faceAppear",
            StairEvent::FaceSeen { .. } => "faceSeen",
            StairEvent::FaceJerk { .. } => "faceJerk",
            StairEvent::FaceGone { .. } => "faceGone",
            StairEvent::FaceBehindArmed => "faceBehindArmed",
            StairEvent::FaceBehind => "faceBehind",
        }
    }
}

/// The current encounter (`StairState::face`).
#[derive(Clone, Debug)]
pub struct FaceState {
    pub kind: FaceKind,
    pub t: f64,
    pub stare: f64,
    pub jerks: i32,
    pub max_jerks: i32,
    pub base: Vec3,
    pub seen: bool,
    /// 'behind': the flat view direction when it arrived
    pub armed: Vec3,
    /// 'behind': time shown
    pub shown_t: f64,
}

/// The descent state (`Stairwell::state()`).
#[derive(Clone, Debug)]
pub struct StairState {
    pub flights: i32,
    pub seg: i32,
    pub x: f64,
    pub z: f64,
    pub y: f64,
    pub eye_y: f64,
    pub step_phase: f64,
    pub deepest_seg: i32,
    pub face: Option<FaceState>,
    pub face_count: i32,
    pub behind_done: bool,
    pub finished: bool,
    pub encounter_at: Vec<i32>,
    pub time: f64,
    pub events: Vec<StairEvent>,
}

/// `update(dt, inp)`'s input: the movement axes, speed and view angles, plus
/// the camera's position and world direction (−Z for a camera) at the time of
/// the call.
#[derive(Clone, Copy, Debug, Default)]
pub struct StairInput {
    pub fwd: f64,
    pub strafe: f64,
    pub speed: f64,
    pub yaw: f64,
    pub pitch: f64,
    pub camera_pos: Vec3,
    pub camera_dir: Vec3,
}
impl StairInput {
    /// Fill the camera fields from a camera object.
    pub fn with_camera(fwd: f64, strafe: f64, speed: f64, yaw: f64, pitch: f64, camera: &Obj) -> StairInput {
        StairInput { fwd, strafe, speed, yaw, pitch, camera_pos: camera.pos(), camera_dir: camera_world_direction(camera) }
    }
}
/// An object's world direction: +Z of its world matrix, negated for cameras.
pub fn camera_world_direction(o: &Obj) -> Vec3 {
    o.world_direction()
}

/// What `update()` returns.
#[derive(Clone, Debug)]
pub struct StairOut {
    pub x: f64,
    /// eye height (`StairState::eye_y`)
    pub y: f64,
    pub z: f64,
    pub seg: i32,
    pub near_exit: bool,
    pub events: Vec<StairEvent>,
}

/// One period of the shaft: its group, the left and right flights and the far and near landings.
#[derive(Clone, Debug)]
pub struct StairChunk {
    pub group: Obj,
    pub left: Obj,
    pub right: Obj,
    pub far: Obj,
    pub near: Obj,
}

/// A point on a landing, from `landing_point` (`far` = the landing at the flights' far end).
#[derive(Clone, Copy, Debug)]
pub struct LandingPoint {
    pub x: f64,
    pub z: f64,
    pub y: f64,
    pub far: bool,
}

/// The Stairwell: the shaft, the Face and the descent state.
pub struct Stairwell {
    /// the shaft's root group (hidden until `enter`)
    pub group: Obj,
    /// the Face mesh
    pub face: Obj,
    /// the descent state
    pub st: StairState,
    pub chunks: Vec<StairChunk>,
    pub top: Obj,
    pub bottom: Obj,
    pub exit_door: Obj,
    pub exit_glow: Obj,
    pub exit_handle: Obj,
    pub floor: Obj,
    pub leds: Vec<Obj>,
    /// whether each LED blinks
    pub led_blink: Vec<bool>,
    pub puddle: Obj,
    pub gouge: Obj,
    pub hole: Obj,
    pub hole_light: Obj,
    led_on: Mat,
    led_off: Mat,
    rng: SharedRng,
}

#[inline]
fn truthy(v: f64) -> bool {
    v != 0.0 && !v.is_nan()
}

#[inline]
fn flight_x(odd: bool) -> f64 {
    (if odd { 1.0 } else { -1.0 }) * (SPINE + (HALF - SPINE) / 2.0)
}

/// The tread height under (x, z) in a segment.
pub fn seg_y(seg: i32, _x: f64, z: f64) -> f64 {
    if seg % 2 != 0 {
        // landing after flight k
        let k = (seg - 1) / 2;
        return -((k + 1) as f64) * DROP;
    }
    let k = seg / 2;
    let odd = k % 2 != 0;
    let along = if odd { FL - z } else { z };
    let i = 0f64.max(((STEPS - 1) as f64).min((along / RUN).floor()));
    -(k as f64) * DROP - (i + 1.0) * RISE
}

pub fn landing_point(landing_seg: i32, on_wall: f64) -> LandingPoint {
    let k = (landing_seg - 1) / 2;
    let far = k % 2 == 0;
    LandingPoint { x: 0.0, z: if far { FL + R - on_wall } else { -R + on_wall }, y: -((k + 1) as f64) * DROP, far }
}

impl Stairwell {
    // ------------------------------------------------------------ build
    pub fn new(deps: StairDeps) -> Stairwell {
        let rng = SharedRng::new(deps.rng);
        let canvas_tex = deps.canvas_tex.clone();
        let root = group();
        root.set_visible(false);

        // ------------------------------------------------------ materials ---
        let tex = |w: f64, h: f64, draw: &mut dyn FnMut(&mut Canvas, f64, f64), rx: f64, ry: f64| -> Tex {
            let t = (*canvas_tex)(w, h, true, draw);
            t.t().repeat.set(rx, ry);
            t
        };
        let concrete_draw = |rng: &SharedRng, base: &str, ctx: &mut Canvas, w: f64, h: f64| {
            ctx.set_fill_style(base);
            ctx.fill_rect(0, 0, w, h);
            for _ in 0..2600 {
                let c = if rng.next() < 0.5 { "0,0,0" } else { "255,255,255" };
                let a = rng.range(0.02, 0.08);
                ctx.set_fill_style(format!("rgba({},{})", c, a));
                let (x, y, ww, hh) = (rng.next() * w, rng.next() * h, rng.range(1.0, 3.0), rng.range(1.0, 3.0));
                ctx.fill_rect(x, y, ww, hh);
            }
            for _ in 0..18 {
                let x = rng.next() * w;
                let y = rng.next() * h;
                let r = rng.range(10.0, 60.0);
                let g = ctx.create_radial_gradient(x, y, 1, x, y, r);
                g.add_color_stop(0, &format!("rgba(20,16,10,{})", rng.range(0.08, 0.2)));
                g.add_color_stop(1, "rgba(0,0,0,0)");
                ctx.set_fill_style(&g);
                ctx.fill_rect(x - r, y - r, r * 2.0, r * 2.0);
            }
        };
        let wall_tex = {
            let r2 = rng.clone();
            tex(256.0, 256.0, &mut |ctx: &mut Canvas, w: f64, h: f64| concrete_draw(&r2, "#5d5a54", ctx, w, h), 2.0, 3.0)
        };
        let step_tex = {
            let r2 = rng.clone();
            tex(
                128.0,
                128.0,
                &mut |ctx: &mut Canvas, w: f64, h: f64| {
                    concrete_draw(&r2, "#6d6a63", ctx, w, h);
                    // worn nosing: a paler band along one edge
                    let g = ctx.create_linear_gradient(0, 0, 0, h * 0.25);
                    g.add_color_stop(0, "rgba(200,195,180,0.28)");
                    g.add_color_stop(1, "rgba(0,0,0,0)");
                    ctx.set_fill_style(&g);
                    ctx.fill_rect(0, 0, w, h * 0.25);
                },
                1.0,
                1.0,
            )
        };
        let wall_mat = lambert!({ map: &wall_tex, side: DoubleSide });
        let step_mat = lambert!({ map: &step_tex });
        let slab_mat = lambert!({ map: &wall_tex });
        let rail_mat = lambert!({ color: 0x3b3a36 });
        let door_mat = lambert!({ color: 0x4d5357 });
        let dark_mat = basic_mat!({ color: 0x000000 });

        // ---------------------------------------------------------- chunk ---
        // one period of the shaft: left flight, far landing, right flight,
        // near landing, and the walls around them for 2·DROP of height
        let hh = DROP * 2.0 + 0.6;
        let step_geo = box_geo!(HALF - SPINE, 0.2, RUN + 0.02);
        let slab_len = hypot2(FL, DROP);
        let under_geo = box_geo!(HALF - SPINE, 0.18, slab_len);
        let land_geo_far = cyl_geo!(R, R, 0.22, 28, 1, false, -PI / 2.0, PI);
        let land_geo_near = cyl_geo!(R, R, 0.22, 28, 1, false, PI / 2.0, PI);
        let curve_far = cyl_geo!(R + 0.02, R + 0.02, hh, 28, 1, true, -PI / 2.0, PI);
        let curve_near = cyl_geo!(R + 0.02, R + 0.02, hh, 28, 1, true, PI / 2.0, PI);
        let side_geo = plane_geo!(FL, hh);
        let spine_geo = box_geo!(SPINE * 2.0, hh, FL);
        let rail_geo = cyl_geo!(0.022, 0.022, slab_len, 6);

        let build_flight = |odd: bool| -> Obj {
            let g = group();
            let steps = instanced_mesh(&step_geo, &step_mat, STEPS as usize);
            let mut m = Mat4::IDENTITY;
            for i in 0..STEPS {
                let along = (i as f64 + 0.5) * RUN;
                let z = if odd { FL - along } else { along };
                m.make_translation(flight_x(odd), -((i + 1) as f64) * RISE - 0.1, z);
                steps.set_matrix_at(i as usize, &m);
            }
            g.add(&steps);
            // sloped slab under the flight (the ceiling of the flight below)
            let under = mesh(&under_geo, &slab_mat);
            let ang = DROP.atan2(FL);
            under.position().set(flight_x(odd), -DROP / 2.0 - 0.34, FL / 2.0);
            under.rotation().x = if odd { -ang } else { ang };
            g.add(&under);
            // handrail along the outer wall, 0.9 m over the nosings
            let rail = mesh(&rail_geo, &rail_mat);
            rail.position().set((if odd { 1.0 } else { -1.0 }) * (HALF - 0.07), -DROP / 2.0 + 0.9, FL / 2.0);
            rail.rotation().x = PI / 2.0 + if odd { -ang } else { ang };
            g.add(&rail);
            g
        };
        let build_chunk = || -> StairChunk {
            let c = group();
            let left = build_flight(false);
            let right = build_flight(true);
            right.position().y = -DROP;
            c.add(&left);
            c.add(&right);
            let far = mesh(&land_geo_far, &step_mat);
            far.position().set(0, -DROP - 0.11, FL);
            c.add(&far);
            let near = mesh(&land_geo_near, &step_mat);
            near.position().set(0, -2.0 * DROP - 0.11, 0);
            c.add(&near);
            // walls for the whole period
            let y_mid = -DROP + 0.3;
            let cf = mesh(&curve_far, &wall_mat);
            cf.position().set(0, y_mid, FL);
            c.add(&cf);
            let cn = mesh(&curve_near, &wall_mat);
            cn.position().set(0, y_mid, 0);
            c.add(&cn);
            for s in [-1.0f64, 1.0] {
                let w = mesh(&side_geo, &wall_mat);
                w.position().set(s * HALF, y_mid, FL / 2.0);
                w.rotation().y = s * -PI / 2.0;
                c.add(&w);
            }
            let sp = mesh(&spine_geo, &wall_mat);
            sp.position().set(0, y_mid, FL / 2.0);
            c.add(&sp);
            StairChunk { group: c, left, right, far, near }
        };
        let chunks: Vec<StairChunk> = (0..4)
            .map(|_| {
                let c = build_chunk();
                root.add(&c.group);
                c
            })
            .collect();

        // top piece: the landing inside the door you came through, a ceiling,
        // and a wall where a flight up would be
        let top = group();
        {
            let land = mesh(&land_geo_near, &step_mat);
            land.position().set(0, -0.11, 0);
            top.add(&land);
            // a parapet across the right-hand mouth: no flight goes up from
            // here, but you can lean over it and look down the well
            let parapet = mesh(box_geo!(HALF - SPINE, 1.05, 0.12), &wall_mat);
            parapet.position().set(SPINE + (HALF - SPINE) / 2.0, 0.52, 0.06);
            top.add(&parapet);
            let ceil = mesh(box_geo!(HALF * 2.0 + 0.2, 0.2, FL + R * 2.0 + 0.2), &slab_mat);
            ceil.position().set(0, 2.7, FL / 2.0);
            top.add(&ceil);
            let curve = mesh(cyl_geo!(R + 0.02, R + 0.02, 3, 28, 1, true, PI / 2.0, PI), &wall_mat);
            curve.position().set(0, 1.4, 0);
            top.add(&curve);
            let curve_f = mesh(cyl_geo!(R + 0.02, R + 0.02, 3, 28, 1, true, -PI / 2.0, PI), &wall_mat);
            curve_f.position().set(0, 1.4, FL);
            top.add(&curve_f);
            for s in [-1.0f64, 1.0] {
                let w = mesh(plane_geo!(FL, 3), &wall_mat);
                w.position().set(s * HALF, 1.4, FL / 2.0);
                w.rotation().y = s * -PI / 2.0;
                top.add(&w);
            }
            let sp = mesh(box_geo!(SPINE * 2.0, 3, FL), &wall_mat);
            sp.position().set(0, 1.4, FL / 2.0);
            top.add(&sp);
            // the door behind you: steel, disguised as a janitor's closet
            let door = mesh(box_geo!(0.95, 2.05, 0.08), &door_mat);
            door.position().set(0, 1.02, -R + 0.05);
            top.add(&door);
            let strip = mesh(plane_geo!(0.9, 0.03), basic_mat!({ color: 0xfff1c8 }));
            strip.position().set(0, 0.015, -R + 0.1);
            strip.rotation().x = -PI / 2.0;
            top.add(&strip); // light under the door
            root.add(&top);
        }

        // bottom piece: the final landing's door, and a floor under it
        let bottom = group();
        let exit_door = mesh(box_geo!(0.95, 2.05, 0.08), &door_mat);
        let exit_glow = mesh(plane_geo!(0.88, 0.035), basic_mat!({ color: 0xd9f0c8 }));
        let exit_handle = mesh(box_geo!(0.12, 0.03, 0.05), &rail_mat);
        let floor = mesh(box_geo!(HALF * 2.0 + 0.4, 0.3, FL + R * 2.0 + 0.4), &slab_mat);
        bottom.add(&exit_door);
        bottom.add(&exit_glow);
        bottom.add(&exit_handle);
        bottom.add(&floor);
        root.add(&bottom);

        // -------------------------------------------------------- details ---
        // An earlier expedition left blue markers on some landings; most are dead.
        let led_geo = sphere_geo!(0.03, 8, 6);
        let led_on = basic_mat!({ color: 0x4fa8ff });
        let led_off = lambert!({ color: 0x1a2430 });
        let leds: Vec<Obj> = (0..4)
            .map(|_| {
                let m = mesh(&led_geo, &led_on);
                m.set_visible(false);
                root.add(&m);
                m
            })
            .collect();
        let puddle = mesh(circle_geo!(0.55, 20), lambert!({ color: 0x1a1812, emissive: 0x020202 }));
        puddle.rotation().x = -PI / 2.0;
        puddle.set_visible(false);
        root.add(&puddle);
        let gouge = mesh(torus_geo!(0.25, 0.02, 4, 16, PI * 0.8), lambert!({ color: 0x9aa6a8, emissive: 0x0c1012 }));
        gouge.rotation().x = -PI / 2.0;
        gouge.set_visible(false);
        root.add(&gouge);
        let hole = mesh(circle_geo!(0.5, 20), &dark_mat);
        hole.set_visible(false);
        root.add(&hole);
        let hole_light = mesh(sphere_geo!(0.012, 6, 4), basic_mat!({ color: 0xffe0a0 }));
        hole_light.set_visible(false);
        root.add(&hole_light);

        // ----------------------------------------------------------- face ---
        let face = match &deps.face_geometry {
            Some(fg) => {
                // a face shell sculpted in Blender with its colours painted in;
                // the emissive is weighted by the same colours (the patch), so
                // it is brightest in the middle and gone at the edge
                let painted = !fg.r().color.is_empty();
                let mat = lambert!({ color: if painted { 0xffffff } else { 0xd8d2c4 }, emissive: if painted { 0x5c574c } else { 0x3e3a33 }, vertexColors: true, fog: false });
                // shader patch: total emissive radiance = emissive × vertex colour
                mat.m().patch = Patch::Stairwell { kind: 0 };
                let m = mesh(fg, &mat);
                m.set_visible(false);
                root.add(&m);
                m
            }
            None => {
                let geo = Self::procedural_face();
                // fog: false — it hangs pale in the dark past where the lamp reaches
                let mat = lambert!({ color: 0xd8d2c4, emissive: 0x4a453c, vertexColors: true, fog: false });
                let m = mesh(&geo, &mat);
                m.set_visible(false);
                root.add(&m);
                m
            }
        };

        let st = StairState {
            flights: 10,
            seg: -1,
            x: 0.6,
            z: -0.6,
            y: 0.0,
            eye_y: 1.62,
            step_phase: 0.0,
            deepest_seg: -1,
            face: None,
            face_count: 0,
            behind_done: false,
            finished: false,
            encounter_at: vec![],
            time: 0.0,
            events: vec![],
        };
        Stairwell {
            group: root,
            face,
            st,
            chunks,
            top,
            bottom,
            exit_door,
            exit_glow,
            exit_handle,
            floor,
            leds,
            led_blink: vec![false; 4],
            puddle,
            gouge,
            hole,
            hole_light,
            led_on,
            led_off,
            rng,
        }
    }

    /// The procedural Face: pale, head-sized, smooth where a mouth, nostrils
    /// and pupils should be.
    fn procedural_face() -> Geo {
        let geo = sphere_geo!(0.15, 48, 36);
        {
            let mut g = geo.g();
            let count = g.vertex_count();
            let mut col = vec![0f32; count * 3];
            for i in 0..count {
                let mut v = Vec3::new(g.position[i * 3] as f64, g.position[i * 3 + 1] as f64, g.position[i * 3 + 2] as f64);
                v.normalize();
                let mut r = 0.15;
                r *= 1.0 + 0.18 * 0f64.max(v.y) - 0.06 * 0f64.max(-v.y); // taller cranium, narrow chin
                let front = 0f64.max(v.z);
                // brow ridge
                let by = v.y - 0.32;
                r += 0.012 * front * (-(by * by) / 0.006).exp();
                // eye hollows: shallow, no eyes in them
                let mut shade = 1.0;
                for s in [-1.0f64, 1.0] {
                    let ex = v.x - s * 0.33;
                    let ey = v.y - 0.16;
                    let d2 = ex * ex + ey * ey * 1.6;
                    let h = (-d2 / 0.012).exp() * front;
                    r -= 0.02 * h;
                    shade -= 0.55 * h;
                }
                // cheekbones, then a smooth sealed plane where the mouth would be
                for s in [-1.0f64, 1.0] {
                    let cx = v.x - s * 0.42;
                    let cy = v.y + 0.08;
                    r += 0.008 * front * (-(cx * cx + cy * cy) / 0.01).exp();
                }
                let my = v.y + 0.35;
                r -= 0.006 * front * (-(v.x * v.x + my * my) / 0.02).exp();
                let back = 0f64.max(-v.z);
                r *= 1.0 - 0.28 * (back * back); // flattened back
                g.position[i * 3] = (v.x * r * 0.86) as f32;
                g.position[i * 3 + 1] = (v.y * r * 1.08) as f32;
                g.position[i * 3 + 2] = (v.z * r * 0.78) as f32;
                col[i * 3] = (shade * 0.92) as f32;
                col[i * 3 + 1] = (shade * 0.9) as f32;
                col[i * 3 + 2] = (shade * 0.86) as f32;
            }
            g.color = col;
            g.color_size = 3;
            g.touch();
        }
        geo.compute_vertex_normals();
        geo
    }

    // ------------------------------------------------------------ queries
    /// Where can a body of radius r stand, and
    /// which segment is it in? The new segment, or `None` if blocked.
    pub fn locate(&self, seg: i32, x: f64, z: f64, r: f64) -> Option<i32> {
        let n = self.st.flights;
        let is_landing = seg % 2 != 0;
        if !is_landing {
            let k = seg / 2;
            let odd = k % 2 != 0;
            let x0 = if odd { SPINE + r } else { -HALF + r };
            let x1 = if odd { HALF - r } else { -SPINE - r };
            if (0.0..=FL).contains(&z) {
                return if x >= x0 && x <= x1 { Some(seg) } else { None };
            }
            // stepping off either end onto a landing
            let down_end = if odd { z < 0.0 } else { z > FL };
            let next = if down_end { seg + 1 } else { seg - 1 };
            return if self.locate(next, x, z, r).is_some() { Some(next) } else { None };
        }
        let k = (seg - 1) / 2;
        let far = k % 2 == 0;
        let on_disc = if far { z >= FL } else { z <= 0.0 };
        if on_disc {
            let cz = if far { FL } else { 0.0 };
            let dz = z - cz;
            let rr = R - r;
            return if x * x + dz * dz <= rr * rr { Some(seg) } else { None };
        }
        // leaving the landing into a flight: which side decides up or down
        let up_side = if far { x < 0.0 } else { x > 0.0 };
        let target = if up_side { seg - 1 } else { seg + 1 };
        let tk = target / 2;
        if tk < 0 || tk > n - 1 {
            return None;
        } // no flight above the door / below the last landing
        if x.abs() < SPINE + r {
            return None;
        } // the spine wall
        if self.locate(target, x, z, r) == Some(target) { Some(target) } else { None }
    }
    pub fn seg_y(&self, seg: i32, x: f64, z: f64) -> f64 {
        seg_y(seg, x, z)
    }
    /// the landing the player is on (on a flight: the flight's index; 0 at the top)
    pub fn landing(&self) -> i32 {
        if self.st.seg % 2 != 0 { (self.st.seg + 1) / 2 } else { 0.max(self.st.seg / 2) }
    }
    /// the depth below the door, in metres
    pub fn depth_m(&self) -> f64 {
        -self.st.y
    }
    /// the number of flights in this descent
    pub fn flights(&self) -> i32 {
        self.st.flights
    }
    /// the descent state
    pub fn state(&self) -> &StairState {
        &self.st
    }
    pub fn state_mut(&mut self) -> &mut StairState {
        &mut self.st
    }
    /// Always the same distance straight down from you — the
    /// child is ~200 m below and never gets any nearer
    pub fn voice_pos(&self) -> Vec3 {
        Vec3::new(0.0, self.st.y - 30.0, FL / 2.0)
    }
    pub fn landing_point(&self, landing_seg: i32, on_wall: f64) -> LandingPoint {
        landing_point(landing_seg, on_wall)
    }

    /// Recycle the four chunks around the player.
    pub fn place_chunks(&self) {
        let pc = 0.max(0.max(self.st.seg) / 4);
        for (i, c) in self.chunks.iter().enumerate() {
            let ci = pc - 1 + i as i32;
            let ok = ci >= 0 && 2 * ci < self.st.flights;
            c.group.set_visible(ok);
            if !ok {
                continue;
            }
            c.group.position().y = -(ci as f64) * 2.0 * DROP;
            let has_right = 2 * ci < self.st.flights - 1;
            c.right.set_visible(has_right);
            c.near.set_visible(has_right);
        }
        self.top.set_visible(pc <= 1);
    }

    // ------------------------------------------------------------ lifecycle
    pub fn enter(&mut self, flights: i32) {
        let rng = self.rng.clone();
        let st = &mut self.st;
        st.flights = 2.max(flights);
        st.seg = -1;
        st.x = 0.55;
        st.z = -0.5;
        st.y = 0.0;
        st.eye_y = 1.62;
        st.step_phase = 0.0;
        st.deepest_seg = -1;
        st.face_count = 0;
        st.behind_done = false;
        st.finished = false;
        st.time = 0.0;
        st.face = None;
        self.face.set_visible(false);
        st.events.clear();
        // encounters get more frequent the deeper the descent goes
        let l = st.flights;
        let a = [(l as f64 * 0.28).floor() as i32, (l as f64 * 0.58).floor() as i32, (l as f64 * 0.86).floor() as i32];
        st.encounter_at =
            a.iter().enumerate().filter(|&(ref i, &f)| f >= 2 && a.iter().position(|&g| g == f) == Some(*i)).map(|(_, &f)| f).collect();
        // the final landing and its door
        let last = landing_point(2 * (l - 1) + 1, 0.06);
        self.bottom.position().set(0, 0, 0);
        self.exit_door.position().set(0, last.y + 1.02, last.z);
        self.exit_door.rotation().y = if last.far { PI } else { 0.0 };
        self.exit_glow.position().set(0, last.y + 0.02, if last.far { last.z - 0.05 } else { last.z + 0.05 });
        self.exit_glow.rotation().x = -PI / 2.0;
        self.exit_handle.position().set(0.32, last.y + 1.0, if last.far { last.z - 0.06 } else { last.z + 0.06 });
        self.floor.position().set(0, last.y - 0.4 - DROP * 0.3, FL / 2.0);
        // markers: LEDs every tenth landing, a puddle, a gouge, a hole with a light in it
        let mut led_landings = vec![];
        let mut ll = 9;
        while ll < l - 1 {
            led_landings.push(ll);
            ll += 10;
        }
        for (i, m) in self.leds.iter().enumerate() {
            let lv = led_landings.get(i).copied();
            m.set_visible(lv.is_some());
            let Some(lv) = lv else { continue };
            let p = landing_point(2 * lv + 1, 0.12);
            let x = rng.range(-0.6, 0.6);
            m.position().set(x, p.y + 0.03, p.z);
            let on = rng.next() < 0.35;
            m.set_mat(if on { &self.led_on } else { &self.led_off });
            self.led_blink[i] = on;
            m.user().set("blink", on);
        }
        let put = |obj: &Obj, landing: i32, dy: f64, on_wall: f64| -> Option<LandingPoint> {
            obj.set_visible(landing > 1 && landing < l - 1);
            if !obj.visible() {
                return None;
            }
            let p = landing_point(2 * landing + 1, on_wall);
            let x = rng.range(-0.4, 0.4);
            obj.position().set(x, p.y + dy, p.z);
            Some(p)
        };
        put(&self.puddle, 16.min(l - 3), 0.012, 0.8);
        put(&self.gouge, (l as f64 * 0.7).floor() as i32, 0.01, 0.9);
        let hp = put(&self.hole, (l as f64 * 0.84).floor() as i32, 1.2, 0.02);
        if let Some(hp) = hp {
            self.hole.rotation().y = if hp.far { PI } else { 0.0 };
            let hpos = self.hole.pos();
            self.hole_light.position().copy(&hpos);
            self.hole_light.position().z += if hp.far { 0.5 } else { -0.5 };
            self.hole_light.position().y -= 0.1;
            self.hole_light.set_visible(true);
        } else {
            self.hole_light.set_visible(false);
        }
        self.place_chunks();
        self.group.set_visible(true);
    }

    /// On the landing at the far end of the flight below
    /// the player, at head height, looking up the flight.
    pub fn spawn_face(&mut self, kind: FaceKind) -> bool {
        let st = &mut self.st;
        let landing_seg = (if st.seg % 2 != 0 { st.seg } else { st.seg + 1 }) + 2;
        if (landing_seg - 1) / 2 > st.flights - 1 {
            return false;
        }
        let p = landing_point(landing_seg, 0.55);
        let max_jerks = if st.face_count == 0 { 1 } else { 2 };
        let bx = self.rng.range(-0.5, 0.5);
        let f = FaceState {
            kind,
            t: 0.0,
            stare: 0.0,
            jerks: 0,
            max_jerks,
            base: Vec3::new(bx, p.y + 1.55, p.z),
            seen: false,
            armed: Vec3::ZERO,
            shown_t: 0.0,
        };
        self.face.position().copy(&f.base);
        st.face = Some(f);
        self.face.set_visible(true);
        st.face_count += 1;
        true
    }
    /// (the caller drops `st.face`)
    fn vanish(&mut self, reason: GoneReason) {
        self.face.set_visible(false);
        self.st.events.push(StairEvent::FaceGone { reason });
        self.st.face = None;
    }
    fn try_move(&mut self, nx: f64, nz: f64) -> bool {
        match self.locate(self.st.seg, nx, nz, 0.3) {
            Some(s) => {
                self.st.seg = s;
                self.st.x = nx;
                self.st.z = nz;
                true
            }
            None => false,
        }
    }

    pub fn update(&mut self, dt: f64, inp: &StairInput) -> StairOut {
        self.st.time += dt;
        // --- movement in plan, segment by segment ---
        let fx = -inp.yaw.sin();
        let fz = -inp.yaw.cos();
        let rx = inp.yaw.cos();
        let rz = -inp.yaw.sin();
        let dx = (fx * inp.fwd + rx * inp.strafe) * inp.speed * dt;
        let dz = (fz * inp.fwd + rz * inp.strafe) * inp.speed * dt;
        if truthy(dx) || truthy(dz) {
            let (x, z) = (self.st.x, self.st.z);
            if !self.try_move(x + dx, z + dz) && !self.try_move(x + dx, z) {
                self.try_move(x, z + dz);
            }
            self.st.step_phase += hypot2(dx, dz);
        }
        if self.st.seg > self.st.deepest_seg {
            self.st.deepest_seg = self.st.seg;
        }
        let foot_y = seg_y(self.st.seg, self.st.x, self.st.z);
        self.st.y += (foot_y - self.st.y) * 1f64.min(dt * 14.0);
        self.place_chunks();
        // LEDs blink like the ones left behind by the last people down here
        for (i, m) in self.leds.iter().enumerate() {
            if m.visible() && self.led_blink[i] {
                let off = ((self.st.time * 1.2 + m.pos().y).floor() % 3.0) == 0.0;
                m.set_mat(if off { &self.led_off } else { &self.led_on });
            }
        }
        // something far down the hole switches a light on and off, two seconds each
        if self.hole_light.visible() {
            let on = truthy((self.st.time / 2.0).floor() % 2.0);
            self.hole_light.mat().m().color.set_scalar(if on { 0.08 } else { 1.0 });
        }

        // --- the Face ---
        let st = &self.st;
        let landing_idx = if st.seg % 2 != 0 { ((st.seg - 1) / 2) as f64 } else { st.seg as f64 / 2.0 - 0.5 };
        if st.face.is_none() && !st.encounter_at.is_empty() && landing_idx >= st.encounter_at[0] as f64 {
            self.st.encounter_at.remove(0);
            if self.spawn_face(FaceKind::Below) {
                self.st.events.push(StairEvent::FaceAppear);
            }
        }
        // turning back: more than a flight above your deepest point
        let cam_pos = inp.camera_pos;
        let st = &self.st;
        if st.face.is_none() && !st.behind_done && st.deepest_seg >= 6 && st.seg <= st.deepest_seg - 3 {
            self.st.behind_done = true;
            let mut d = inp.camera_dir;
            d.y = 0.0;
            d.normalize();
            self.st.face = Some(FaceState {
                kind: FaceKind::Behind,
                t: 0.0,
                stare: 0.0,
                jerks: 0,
                max_jerks: 0,
                base: Vec3::ZERO,
                seen: false,
                armed: d,
                shown_t: 0.0,
            });
            self.face.set_visible(false);
            self.st.events.push(StairEvent::FaceBehindArmed);
        }
        if let Some(mut f) = self.st.face.take() {
            // `f` deliberately lives on after a vanish, so later checks in the
            // same frame can still fire; it is only written back while it is
            // still the current face.
            let mut alive = true;
            f.t += dt;
            let face_pos = self.face.pos();
            let mut to_face = Vec3::ZERO;
            to_face.sub_vectors(&face_pos, &cam_pos);
            let dist = to_face.length();
            let cam_dir = inp.camera_dir;
            let in_view = to_face.normalized().dot(&cam_dir) > 0.5f64.cos();
            match f.kind {
                FaceKind::Below => {
                    self.face.look_at(cam_pos.x, cam_pos.y, cam_pos.z);
                    self.face.rotate_z(if f.jerks == 1 {
                        0.026
                    } else if f.jerks == 2 {
                        -0.02
                    } else {
                        0.0
                    }); // each jerk leaves it at a slightly different roll
                    if in_view && dist < 9.0 {
                        if !f.seen {
                            f.seen = true;
                            self.st.events.push(StairEvent::FaceSeen { dist });
                        }
                        f.stare += dt;
                        // it stares, sometimes for as long as thirteen seconds, then jerks
                        let limit = if f.jerks == 0 { self.rng.range(2.5, 6.0) } else { self.rng.range(1.5, 3.0) };
                        if f.stare > limit && f.jerks < f.max_jerks {
                            f.jerks += 1;
                            f.stare = 0.0;
                            let step = if f.jerks == 1 { 0.5 } else { 0.9 };
                            let mut dir = Vec3::ZERO;
                            dir.sub_vectors(&cam_pos, &self.face.pos());
                            dir.normalize();
                            self.face.position().add_scaled_vector(&dir, step);
                            self.st.events.push(StairEvent::FaceJerk { n: f.jerks, dist: dist - step });
                        } else if f.jerks >= f.max_jerks && f.stare > 1.2 {
                            self.vanish(GoneReason::Stared);
                            alive = false;
                        }
                    }
                    if dist < 1.6 {
                        self.vanish(GoneReason::Approached);
                        alive = false;
                    }
                    if f.t > 40.0 {
                        self.vanish(GoneReason::Timeout);
                        alive = false;
                    }
                }
                FaceKind::Behind => {
                    // It is behind you while you climb. The moment you turn
                    // round — more than 100° from the way you were facing when
                    // it arrived — it is thirty centimetres from your eyes.
                    let flat = Vec3::new(cam_dir.x, 0.0, cam_dir.z).normalized();
                    if !f.seen && flat.dot(&f.armed) < (100.0 * PI / 180.0).cos() {
                        f.seen = true;
                        f.shown_t = 0.0;
                        self.face.position().set(cam_pos.x + flat.x * 0.32, cam_pos.y - 0.02, cam_pos.z + flat.z * 0.32);
                        self.face.look_at(cam_pos.x, cam_pos.y, cam_pos.z);
                        self.face.set_visible(true);
                        self.st.events.push(StairEvent::FaceBehind);
                    }
                    if f.seen {
                        f.shown_t += dt;
                        if f.shown_t > 0.75 {
                            self.vanish(GoneReason::Behind);
                            alive = false;
                        }
                    } else if f.t > 45.0 {
                        self.vanish(GoneReason::Timeout);
                        alive = false;
                    }
                }
            }
            if alive {
                self.st.face = Some(f);
            }
        }

        // --- the door at the bottom ---
        let st = &mut self.st;
        let last = landing_point(2 * (st.flights - 1) + 1, 0.06);
        let near_exit = st.seg == 2 * (st.flights - 1) + 1 && hypot2(st.x - 0.0, st.z - last.z) < 1.2;
        st.eye_y = st.y + 1.62;
        StairOut { x: st.x, y: st.eye_y, z: st.z, seg: st.seg, near_exit, events: std::mem::take(&mut st.events) }
    }

    /// Hide everything and detach the group from its parent (the game itself
    /// keeps the Stairwell for the whole session).
    pub fn dispose(&mut self) {
        self.group.set_visible(false);
        self.face.set_visible(false);
        self.st.face = None;
        self.group.remove_from_parent();
    }
}
