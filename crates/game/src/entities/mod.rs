//! The entities: materials, rigs, animation, steering, look-rules, voices,
//! `update_entities`.
//!
//! The Smiler, the Hound and the Watcher: articulated figures with named
//! joints, driven every frame by `animate_entity()`. The organic parts come
//! from tools/blender/creatures.py (`assets::creatures`), each modelled in its
//! joint's own frame, so the rigs below place them with no fitting. Painted
//! parts carry their albedo as vertex colours (materials are white).
//!
//! Split over three files: this one (materials, shared geometry, the rigs and
//! `make_smiler` / `make_hound` / `make_watcher`, `spawn_entities`, `add_entity`,
//! `update_face`), `anim.rs` (`entity_tics`, `animate_entity`,
//! `apply_watcher_pose`, the hound's limb IK) and `ai.rs` (steering,
//! look-rules, voices, `update_entities`).
//!
//! Per-entity state that `records::Entity` has no typed slot for lives in
//! `e.num` / `e.flag` under string keys (`e.n("poseYaw")`, `e.fl("aimOk")`),
//! and the non-numeric pieces (the typed rig, the aim field, the tic spring,
//! the voice) in `g.ent_ext[e.g.id()]` (`EntExt`). `e.parts` (the
//! `records::Entity` map) holds every named bone by its name plus the rig's
//! named parts (`head`, `jaw`, `torso`, `skin`, ...).
//!
//! CONTRACT: the signatures below are called from other modules; keep them.
pub mod ai;
pub mod body;
pub mod brain;
pub mod corner;
pub mod creature;
pub mod director;
pub mod hound;
pub mod reader;
pub mod smiler;
pub mod watcher;

use crate::base::records::*;
use crate::prelude::*;
use std::collections::HashMap;

/// The kinds that roam the rooms and that the director sends.
pub const ROAMING_KINDS: [&str; 4] = ["smiler", "hound", "watcher", "reader"];
/// The ecology's places: a tile darker than this is the dark (the Smiler's);
/// how many such tiles, furnished ones (the Watcher's) and office wing
/// tiles (the Reader's the more) make a level call for the kind.
pub const DARK: f64 = 0.2;
pub const DARK_TILES: usize = 12;
pub const FURNISHED_TILES: usize = 8;
pub const OFFICE_TILES: usize = 8;

/// What `make_smiler` / `make_hound` / `make_watcher` return.
#[derive(Clone, Debug)]
pub struct EntityBuild {
    pub g: Obj,
    pub face: Option<Obj>,
    pub face_y: f64,
    pub parts: std::collections::HashMap<String, Obj>,
}

pub fn ent_smooth(a: f64, b: f64, x: f64) -> f64 {
    let k = clamp((x - a) / (b - a), 0.0, 1.0);
    k * k * (3.0 - 2.0 * k)
}
pub fn wrap_pi(a: f64) -> f64 {
    let mut a = a;
    while a > PI {
        a -= PI * 2.0;
    }
    while a < -PI {
        a += PI * 2.0;
    }
    a
}

// ------------------------------------------------------------ skin detail
/// Of the canvas textures: draw nine times so the tile wraps.
fn wrap_draw(ctx: &mut Canvas, w: f64, h: f64, mut f: impl FnMut(&mut Canvas)) {
    for dx in [-w, 0.0, w] {
        for dy in [-h, 0.0, h] {
            ctx.save();
            ctx.translate(dx, dy);
            f(ctx);
            ctx.restore();
        }
    }
}

// Skin detail without UVs: pores, fine creases, veins and soft lumps, as a
// bump read along three axes (triplanar) from a tiling height map at each
// vertex's rest-pose position, so the detail stays on the skin while a
// skinned body bends. Used by every creature skin.
crate::lazy_static_tex!(skin_detail_tex, {
    let t = canvas_tex(256, 256, |ctx, w, h| {
        ctx.set_fill_style("#808080");
        ctx.fill_rect(0, 0, w, h);
        // soft lumps: the flesh is not smooth
        for _ in 0..70 {
            let (x, y, r, up) = (rnd() * w, rnd() * h, 8.0 + rnd() * 26.0, rnd() < 0.5);
            wrap_draw(ctx, w, h, |ctx: &mut Canvas| {
                let gr = ctx.create_radial_gradient(x, y, 0, x, y, r);
                gr.add_color_stop(0, if up { "rgba(255,255,255,0.16)" } else { "rgba(0,0,0,0.16)" });
                gr.add_color_stop(1, "rgba(128,128,128,0)");
                ctx.set_fill_style(&gr);
                ctx.begin_path();
                ctx.arc(x, y, r, 0, 7);
                ctx.fill();
            });
        }
        // creases: short, slightly curved grooves, mostly one way like skin lines
        ctx.set_line_cap("round");
        for _ in 0..160 {
            let (x, y) = (rnd() * w, rnd() * h);
            let a = (if rnd() < 0.7 { 0.3 } else { 1.8 }) + (rnd() - 0.5) * 0.7;
            let (l, bend) = (6.0 + rnd() * 22.0, (rnd() - 0.5) * 8.0);
            ctx.set_line_width(0.7 + rnd() * 1.1);
            ctx.set_stroke_style(format!("rgba(0,0,0,{})", 0.25 + rnd() * 0.3));
            wrap_draw(ctx, w, h, |ctx: &mut Canvas| {
                ctx.begin_path();
                ctx.move_to(x, y);
                ctx.quadratic_curve_to(
                    x + a.cos() * l / 2.0 - a.sin() * bend,
                    y + a.sin() * l / 2.0 + a.cos() * bend,
                    x + a.cos() * l,
                    y + a.sin() * l,
                );
                ctx.stroke();
            });
        }
        // veins: long, raised, wandering
        for _ in 0..9 {
            let (mut x, mut y, mut a) = (rnd() * w, rnd() * h, rnd() * 7.0);
            let mut pts = vec![(x, y)];
            for _ in 0..9 {
                a += (rnd() - 0.5) * 0.9;
                x += a.cos() * 11.0;
                y += a.sin() * 11.0;
                pts.push((x, y));
            }
            ctx.set_line_width(1.6 + rnd());
            ctx.set_stroke_style("rgba(255,255,255,0.22)");
            wrap_draw(ctx, w, h, |ctx: &mut Canvas| {
                ctx.begin_path();
                ctx.move_to(pts[0].0, pts[0].1);
                for &(px, py) in &pts {
                    ctx.line_to(px, py);
                }
                ctx.stroke();
            });
        }
        // pores
        for _ in 0..1400 {
            let (x, y, r) = (rnd() * w, rnd() * h, 0.5 + rnd() * 0.9);
            ctx.set_fill_style(format!("rgba(0,0,0,{})", 0.3 + rnd() * 0.35));
            wrap_draw(ctx, w, h, |ctx: &mut Canvas| {
                ctx.begin_path();
                ctx.arc(x, y, r, 0, 7);
                ctx.fill();
            });
        }
    });
    t.t().color_space = ColorSpace::NoColorSpace; // heights, not colours
    t
});

/// The bump map, its strength and the
/// engine's `Patch::SkinDetail` (the shader: triplanar detail at the rest-pose
/// position, `u_detail_scale` = `scale`, `bump_scale` = `strength`, wet in the
/// creases and dull on the lumps). Returns the same material.
pub fn with_skin_detail(mat: Mat, scale: f64, strength: f64) -> Mat {
    let tex = skin_detail_tex();
    {
        let mut m = mat.m();
        m.bump_map = Some(tex.clone());
        m.bump_scale = strength;
        m.patch = Patch::SkinDetail { map: tex, scale, strength, shine: 1.0, rest_space: true };
        m.version += 1;
    }
    mat
}

/// Bind a skinned body to the rig's named bones
/// (the names come with the geometry). Called right after the rig is built,
/// while it still stands in the rest pose the body was modelled in.
pub fn skin_body(root: &Obj, geo: &Geo, mat: &Mat) -> Option<Obj> {
    let mut by_name: HashMap<String, Obj> = HashMap::new();
    root.traverse(&mut |o| {
        if o.is_bone() {
            by_name.insert(o.name(), o.clone());
        }
    });
    let names = geo.r().bone_names.clone();
    let mut bones = Vec::with_capacity(names.len());
    for n in &names {
        bones.push(by_name.get(n)?.clone());
    }
    let sm = skinned_mesh(geo, mat);
    root.add(&sm);
    root.update_matrix_world(true);
    bind_skeleton(&sm, bones);
    // a generous sphere: the animation reaches well past the rest pose (the
    // renderer gives every skinned mesh its bounding radius × 1.35 + 0.3)
    Some(sm)
}

/// A skinned mesh in attached bind mode needs `bind_matrix_inverse` re-derived
/// from its world matrix whenever it moves; the engine keeps the one from
/// `bind()`. The rigs move (their root group carries the position), so this
/// brings it up to date — call after the entity has moved.
pub fn sync_skin_bind(sm: &Obj) {
    sm.update_world_matrix(true, false);
    let inv = sm.matrix_world().inverted();
    if let engine::scene::Kind::Skinned { bind_matrix_inverse, .. } = &mut sm.n().kind {
        *bind_matrix_inverse = inv;
    }
}

/// `watcher_cloth_tex` — the cloth itself at close range: a loose plain weave,
/// slubs in the thread, and the small creases of an old folded sheet.
/// The dust sheet's weave (made once).
pub(crate) fn watcher_cloth_tex() -> Tex {
    thread_local! {
        static T: std::cell::RefCell<Option<Tex>> = const { std::cell::RefCell::new(None) };
    }
    T.with(|t| t.borrow_mut().get_or_insert_with(make_watcher_cloth_tex).clone())
}

fn make_watcher_cloth_tex() -> Tex {
    let t = canvas_tex(256, 256, |ctx, w, h| {
        ctx.set_fill_style("#808080");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..40 {
            // soft unevenness in the cloth
            let (x, y, r, up) = (rnd() * w, rnd() * h, 10.0 + rnd() * 30.0, rnd() < 0.5);
            wrap_draw(ctx, w, h, |ctx: &mut Canvas| {
                let gr = ctx.create_radial_gradient(x, y, 0, x, y, r);
                gr.add_color_stop(0, if up { "rgba(255,255,255,0.1)" } else { "rgba(0,0,0,0.1)" });
                gr.add_color_stop(1, "rgba(128,128,128,0)");
                ctx.set_fill_style(&gr);
                ctx.begin_path();
                ctx.arc(x, y, r, 0, 7);
                ctx.fill();
            });
        }
        ctx.set_line_cap("round");
        for _ in 0..70 {
            // fold-creases: a groove with a lit lip, mostly hanging downward
            let (x, y, a) = (rnd() * w, rnd() * h, PI / 2.0 + (rnd() - 0.5) * 1.6);
            let (l, bend, k) = (12.0 + rnd() * 40.0, (rnd() - 0.5) * 12.0, 0.5 + rnd() * 0.5);
            let (ex, ey) = (x + a.cos() * l, y + a.sin() * l);
            let (mx, my) = ((x + ex) / 2.0 - a.sin() * bend, (y + ey) / 2.0 + a.cos() * bend);
            wrap_draw(ctx, w, h, |ctx: &mut Canvas| {
                ctx.set_line_width(2.2);
                ctx.set_stroke_style(format!("rgba(255,255,255,{})", 0.16 * k));
                ctx.begin_path();
                ctx.move_to(x + 1.2, y);
                ctx.quadratic_curve_to(mx + 1.2, my, ex + 1.2, ey);
                ctx.stroke();
                ctx.set_line_width(1.4);
                ctx.set_stroke_style(format!("rgba(0,0,0,{})", 0.3 * k));
                ctx.begin_path();
                ctx.move_to(x, y);
                ctx.quadratic_curve_to(mx, my, ex, ey);
                ctx.stroke();
            });
        }
        let mut y = 0.0;
        while y < h {
            // the weave: weft and warp, a thread every two pixels
            ctx.set_fill_style(format!("rgba(0,0,0,{})", 0.05 + rnd() * 0.05));
            ctx.fill_rect(0, y, w, 1);
            y += 2.0;
        }
        let mut x = 0.0;
        while x < w {
            ctx.set_fill_style(format!("rgba(255,255,255,{})", 0.04 + rnd() * 0.04));
            ctx.fill_rect(x, 0, 1, h);
            x += 2.0;
        }
        for _ in 0..220 {
            // slubs: thick spots in the thread
            let (x, y, l) = (rnd() * w, (rnd() * h / 2.0).floor() * 2.0, 3.0 + rnd() * 9.0);
            ctx.set_fill_style(format!("rgba(255,255,255,{})", 0.12 + rnd() * 0.12));
            wrap_draw(ctx, w, h, |ctx: &mut Canvas| ctx.fill_rect(x, y, l, 1));
        }
    });
    t.t().color_space = ColorSpace::NoColorSpace; // heights, not colours
    t
}

/// An entity's voice: its soundscape slot, the next call time and whether it was hunting
#[derive(Clone)]
pub struct EntVoice {
    pub slot: crate::audio::soundscape::VoiceSlot,
    pub next: f64,
    pub was_hunting: bool,
}
/// Per-entity state that `records::Entity` has no typed slot for, by
/// `e.g.id()` in `g.ent_ext`.
#[derive(Clone, Default)]
pub struct EntExt {
    /// the aim field (compared by identity)
    pub aim_field: Option<Rc<Vec<i32>>>,
    /// the voice
    pub voice: Option<EntVoice>,
    /// a creature of the redesign: its body and brain (`creature`)
    pub creature: Option<Box<creature::Creature>>,
}

impl Game {
    /// start-up work (creature materials, textures, shared geometry)
    pub fn init_entities(&mut self) {
        self.entities.clear();
        self.player_field = None;
        self.player_field_t = 0.0;
        self.heart_t = 0.0;
    }

    /// The creatures a level of the rooms is given, by its places
    /// (`docs/ENTITIES.md` design 5, the ecology): dark regions call the
    /// Smiler, furnished rooms the Watcher, paper lying about the Reader
    /// (office wings the more), and the Hound goes anywhere; one on the
    /// Lobby (a Hound, or a Smiler where it has a dark region), two at depth
    /// 2, three deeper, each kind once while the places allow. Chosen by the
    /// level, not by chance (no draws). The one in the corner is placed on
    /// its own (`corner::place`).
    pub fn ecology(&self) -> Vec<&'static str> {
        let depth = self.depth();
        // (the Showroom and the Borrowed House keep their own residents; the
        // levels with the Stairwell's or the Loop's door are rooms like any)
        if depth < 1 || self.building.is_some() || matches!(self.level_scp.as_deref(), Some("store" | "house")) {
            return vec![];
        }
        // (places a creature can use: reached from the arrival, a quarter of
        // the level or more from it, as `spawn_entities` puts them)
        let from = self.bfs(1, 1);
        let (mut dark, mut furnished, mut office) = (0, 0, 0);
        for z in 1..self.h - 1 {
            for x in 1..self.w - 1 {
                let d = from.get(self.t_idx(x, z)).copied().unwrap_or(-1);
                if self.is_wall_tile(x, z) || d < 0 || (d as f64) <= self.w as f64 * 0.25 {
                    continue;
                }
                if self.e_region_light_at(x, z) < DARK {
                    dark += 1;
                }
                match self.zone_purpose_at(x, z).map(|p| p.label) {
                    Some("living quarters" | "parlour") => furnished += 1,
                    Some("office wing") => office += 1,
                    _ => {}
                }
            }
        }
        let paper = self.papers.iter().any(|p| !p.taken);
        let mut h = crate::base::rng::SplitMix64::new(self.current_seed, 0xEC01_0617 ^ self.level_num as u64);
        if depth == 1 {
            return vec![if dark >= DARK_TILES && h.next() < 0.5 { "smiler" } else { "hound" }];
        }
        // what the places call for, and how strongly (ties broken by the level)
        let mut want: Vec<(&'static str, f64)> = vec![("hound", 1.0 + h.next())];
        if dark >= DARK_TILES {
            want.push(("smiler", 1.4 + h.next()));
        }
        if furnished >= FURNISHED_TILES {
            want.push(("watcher", 1.2 + h.next()));
        }
        if paper {
            want.push(("reader", if office >= OFFICE_TILES { 1.6 } else { 1.0 } + h.next()));
        }
        want.sort_by(|a, b| b.1.total_cmp(&a.1));
        let count = if depth == 2 { 2 } else { 3 };
        let mut kinds: Vec<&'static str> = want.iter().take(count).map(|k| k.0).collect();
        // fewer places than creatures: another of what goes anywhere
        while kinds.len() < count {
            kinds.push(
                if dark >= DARK_TILES && kinds.iter().filter(|k| **k == "hound").count() > kinds.iter().filter(|k| **k == "smiler").count()
                {
                    "smiler"
                } else {
                    "hound"
                },
            );
        }
        kinds
    }

    /// Put the level's creatures (`ecology`; `count` 0 clears them, as on an
    /// anomaly's level) far from the arrival, each in its own kind of place
    /// where the level has one (the Smiler in the dark, the Watcher among
    /// furniture). Seeded: one draw per pick, then `add_entity`'s heading.
    /// The Reader is placed after the build, beside its paper
    /// (`reader::place`).
    pub fn spawn_entities(&mut self, count: i32, dist_from_spawn: &[i32]) {
        for e in &self.entities {
            if let Some(f) = &e.face {
                self.scene.remove(f);
            }
        } // faces live in scene, not world
        self.entities.clear();
        self.ent_ext.clear();
        if count == 0 {
            return;
        }
        let far = |g: &Game, x: i32, z: i32| g.tiles[g.t_idx(x, z)] == 0 && dist_from_spawn[g.t_idx(x, z)] as f64 > g.w as f64 * 0.45;
        let mut taken: Vec<(i32, i32)> = vec![];
        for (i, kind) in self.ecology().into_iter().filter(|k| *k != "reader").enumerate() {
            let fits = |g: &Game, x: i32, z: i32| match kind {
                "smiler" => g.e_region_light_at(x, z) < DARK,
                "watcher" => matches!(g.zone_purpose_at(x, z).map(|p| p.label), Some("living quarters" | "parlour")),
                _ => true,
            };
            // its own kind of place far from the arrival; else nearer (a
            // quarter of the level off); else anywhere far
            let (mut open, mut nearer, mut anywhere) = (vec![], vec![], vec![]);
            for z in 1..self.h - 1 {
                for x in 1..self.w - 1 {
                    let i = self.t_idx(x, z);
                    if self.tiles[i] != 0 || taken.contains(&(x, z)) {
                        continue;
                    }
                    let d = dist_from_spawn[i] as f64;
                    let fit = fits(self, x, z);
                    if far(self, x, z) {
                        anywhere.push((x, z));
                        if fit {
                            open.push((x, z));
                        }
                    } else if fit && d > self.w as f64 * 0.25 {
                        nearer.push((x, z));
                    }
                }
            }
            let pool = if !open.is_empty() {
                open
            } else if !nearer.is_empty() {
                nearer
            } else {
                anywhere
            };
            if pool.is_empty() {
                break;
            }
            let (x, z) = pool[((rnd() * pool.len() as f64).floor() as usize).min(pool.len() - 1)];
            taken.push((x, z));
            self.add_entity(kind, x, z, i as i32);
            if kind == "watcher" {
                watcher::company(self, x, z);
            }
        }
    }

    pub fn add_entity(&mut self, kind: &str, x: i32, z: i32, i: i32) -> usize {
        // (every kind made here is a creature of the redesign: `creature`)
        let (built, c) = self.make_creature(kind, x as f64, z as f64);
        let mut e = Entity::new(kind, built.g.clone());
        e.face_y = built.face_y;
        e.parts = built.parts;
        // (a seeded draw inside a level's build: the picks after it keep their places)
        e.yaw = rand1(PI * 2.0);
        e.state = "wander".into();
        e.w_field = None;
        e.w_target = None;
        // held still in the level's first seconds, the later ones longer
        e.freeze = 2.0 + i as f64 * 1.5;
        self.ent_ext.entry(e.g.id()).or_default().creature = Some(Box::new(c));
        self.entities.push(e);
        self.entities.len() - 1
    }

    /// Billboard the glowing face just proud of the head,
    /// toward the camera, anchored at the head bone's eye line.
    pub fn update_face(&mut self, ei: usize) {
        let Some(e) = self.entities.get(ei) else { return };
        let Some(face) = e.face.clone() else { return };
        let head_pos = match e.parts.get("head") {
            Some(head) => {
                let hw = head.world_position();
                v3(hw.x, hw.y + e.eye_y_offset, hw.z)
            }
            None => {
                let p = e.pos();
                v3(p.x, e.face_y, p.z)
            }
        };
        let cam = self.camera.pos();
        let mut to_cam = cam;
        to_cam.sub(&head_pos);
        to_cam.normalize();
        let mut p = head_pos;
        p.add_scaled_vector(&to_cam, 0.17);
        face.set_pos_v(&p);
        face.look_at(cam.x, cam.y, cam.z);
    }

    /// Bring every rig's skinned body up to date with where its entity stands
    /// (see `sync_skin_bind`); `update_entities` calls it after moving them.
    pub fn sync_entity_skins(&self) {
        for (i, e) in self.entities.iter().enumerate() {
            if let Some(c) = self.creature(i) {
                c.body.sync();
            } else {
                // (a figure on the human rig: its body and its clothes)
                for (k, sk) in &e.parts {
                    if k == "skin" || k.starts_with("skin.") {
                        sync_skin_bind(sk);
                    }
                }
            }
        }
    }

    // ------------------------------------------------------------ navigation seam
    // The world queries the entities use, through an optional override that
    // tests install (`set_entity_nav`) to run the AI on a known grid with
    // reference grid implementations; without one they are the game's own
    // (world/, visuals/lighting.rs).
    pub(crate) fn e_bfs(&self, sx: i32, sz: i32) -> Vec<i32> {
        match entity_nav() {
            Some(n) => n.bfs(self, sx, sz),
            None => self.bfs(sx, sz),
        }
    }
    pub(crate) fn e_los(&self, ax: f64, az: f64, bx: f64, bz: f64) -> bool {
        match entity_nav() {
            Some(n) => n.los(self, ax, az, bx, bz),
            None => self.los(ax, az, bx, bz),
        }
    }
    pub(crate) fn e_circle_hits_wall(&self, px: f64, pz: f64, r: f64) -> bool {
        match entity_nav() {
            Some(n) => n.circle_hits_wall(self, px, pz, r),
            None => self.circle_hits_wall(px, pz, r),
        }
    }
    pub(crate) fn e_collide_move(&self, p: &mut Vec3, dx: f64, dz: f64, r: f64) {
        match entity_nav() {
            Some(n) => n.collide_move(self, p, dx, dz, r),
            None => self.collide_move(p, dx, dz, r),
        }
    }
    pub(crate) fn e_nav_tile_at(&self, px: f64, pz: f64) -> (i32, i32) {
        match entity_nav() {
            Some(n) => n.nav_tile_at(self, px, pz),
            None => self.nav_tile_at(px, pz),
        }
    }
    pub(crate) fn e_region_light_at(&self, x: i32, z: i32) -> f64 {
        match entity_nav() {
            Some(n) => n.region_light_at(self, x, z),
            None => self.region_light_at(x, z),
        }
    }
}

/// The world queries of the entity AI (`bfs`, `los`, `circle_hits_wall`,
/// `collide_move`, `nav_tile_at`, `ceil_at_world`, `region_light_at`). Tests install
/// one with `set_entity_nav`; the game never does.
pub trait EntityNav {
    fn bfs(&self, g: &Game, sx: i32, sz: i32) -> Vec<i32>;
    fn los(&self, g: &Game, ax: f64, az: f64, bx: f64, bz: f64) -> bool;
    fn circle_hits_wall(&self, g: &Game, px: f64, pz: f64, r: f64) -> bool;
    fn collide_move(&self, g: &Game, p: &mut Vec3, dx: f64, dz: f64, r: f64);
    fn nav_tile_at(&self, g: &Game, px: f64, pz: f64) -> (i32, i32);
    fn ceil_at_world(&self, g: &Game, px: f64, pz: f64) -> f64;
    fn region_light_at(&self, g: &Game, x: i32, z: i32) -> f64;
}
thread_local! { static NAV: RefCell<Option<Rc<dyn EntityNav>>> = const { RefCell::new(None) }; }
/// Install (or clear) the world-query override on this thread (tests).
pub fn set_entity_nav(n: Option<Rc<dyn EntityNav>>) {
    NAV.with(|c| *c.borrow_mut() = n);
}
fn entity_nav() -> Option<Rc<dyn EntityNav>> {
    NAV.with(|c| c.borrow().clone())
}
