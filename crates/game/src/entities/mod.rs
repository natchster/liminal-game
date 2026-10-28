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
pub mod anim;
pub mod body;
pub mod brain;
pub mod corner;
pub mod creature;
pub mod director;
pub mod hound;
pub mod reader;
pub mod smiler;
pub mod watcher;

use crate::assets::creatures::{creature_geometry, watcher_geometry};
use crate::base::numeric::{hypot2, hypot3};
use crate::base::records::*;
use crate::prelude::*;
use std::collections::HashMap;
use std::sync::OnceLock;

/// The kinds that roam the rooms and that the director sends.
pub const ROAMING_KINDS: [&str; 4] = ["smiler", "hound", "watcher", "reader"];

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

/// Two-bone IK in a limb's plane (y up, z forward) → (t1, t2): from the root
/// joint at (hy, hz) to a target (ty, tz) with bones a then b; `fwd` bends the
/// middle joint forward (a knee) or back (an elbow). The absolute bone angles
/// from straight down (+ = forward). `fwd` is a flag as a number: true unless
/// it is 0 or NaN.
pub fn solve_limb(hy: f64, hz: f64, ty: f64, tz: f64, a: f64, b: f64, fwd: f64) -> (f64, f64) {
    let fwd = fwd != 0.0 && !fwd.is_nan();
    let (mut dy, mut dz) = (ty - hy, tz - hz);
    let mut d = hypot2(dy, dz);
    let d_max = (a + b) * 0.998;
    if d > d_max {
        dy *= d_max / d;
        dz *= d_max / d;
        d = d_max;
    }
    d = d.max((a - b).abs() + 1e-3);
    let aa = clamp((a * a + d * d - b * b) / (2.0 * a * d), -1.0, 1.0).acos();
    let t1 = dz.atan2(-dy) + if fwd { aa } else { -aa };
    let t2 = (dz - t1.sin() * a).atan2(-(dy + t1.cos() * a));
    (t1, t2)
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

/// Small, wet, white-hot orbs deep under the brow; the
/// glow falls off towards the lids (`Patch::SmilerEye`). One per Smiler.
pub fn smiler_eye_material() -> Mat {
    let m = phong!({ color: 0x15171c, emissive: 0xffffff, specular: 0xffffff, shininess: 150 });
    m.m().patch = Patch::SmilerEye;
    m
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

// ------------------------------------------------------------ face billboards
/// `SM_GRIN` — the far face's grin follows the Blender head
/// (tools/blender/creatures.py smh_gy / smh_gL).
pub struct SmGrin {
    pub mid: f64,
    pub cor: f64,
    pub amax: f64,
    pub lmid: f64,
    pub lcor: f64,
    pub r: f64,
}
pub const SM_GRIN: SmGrin = SmGrin { mid: -0.09, cor: -0.01, amax: 70.0 * PI / 180.0, lmid: 0.040, lcor: 0.022, r: 0.135 };
/// → [x, band centre (m from the eye line), half height]
pub fn sm_grin_at(a: f64) -> [f64; 3] {
    let u = 1f64.min(a.abs() / SM_GRIN.amax);
    let mut l = SM_GRIN.lmid + (SM_GRIN.lcor - SM_GRIN.lmid) * 1f64.min(u / 0.85).powf(1.5);
    if u > 0.85 {
        l *= 1.0 - 0.72 * ((u - 0.85) / 0.15).powf(1.5);
    }
    [SM_GRIN.r * a.sin(), SM_GRIN.mid + (SM_GRIN.cor - SM_GRIN.mid) * u.powf(1.8), l / 2.0]
}
fn make_smiler_face_tex() -> Tex {
    canvas_tex_opt(256, 256, false, |ctx, _w, _h| {
        ctx.clear_rect(0, 0, 256, 256);
        let k = 256.0 / 0.6;
        let px = |x: f64| 128.0 + x * k;
        let py = |y: f64| 128.0 - y * k; // a 0.6 m plane
        let halo = ctx.create_radial_gradient(128, 140, 6, 128, 140, 118);
        halo.add_color_stop(0, "rgba(238,242,255,0.22)");
        halo.add_color_stop(0.55, "rgba(238,242,255,0.05)");
        halo.add_color_stop(1, "rgba(238,242,255,0)");
        ctx.set_fill_style(&halo);
        ctx.fill_rect(0, 0, 256, 256);
        ctx.set_shadow_color("#eef2ff");
        ctx.set_shadow_blur(34);
        ctx.set_fill_style("#ffffff");
        // two small round eyes, 0.094 m apart like the head's, on the canvas midline (the anchor)
        ctx.begin_path();
        ctx.ellipse(108, 128, 8, 8, 0, 0, 7);
        ctx.fill();
        ctx.begin_path();
        ctx.ellipse(148, 128, 8, 8, 0, 0, 7);
        ctx.fill();
        // the crescent, rising to the eye line at +/-70 degrees round the face
        ctx.set_shadow_blur(14);
        ctx.begin_path();
        for i in 0..=40 {
            let [x, y, h] = sm_grin_at(-SM_GRIN.amax + 2.0 * SM_GRIN.amax * i as f64 / 40.0);
            if i != 0 {
                ctx.line_to(px(x), py(y + h));
            } else {
                ctx.move_to(px(x), py(y + h));
            }
        }
        for i in (0..=40).rev() {
            let [x, y, h] = sm_grin_at(-SM_GRIN.amax + 2.0 * SM_GRIN.amax * i as f64 / 40.0);
            ctx.line_to(px(x), py(y - h));
        }
        ctx.close_path();
        ctx.fill();
        // notches between thirty interlocking teeth
        ctx.set_shadow_blur(0);
        ctx.set_global_composite_operation("destination-out");
        ctx.set_line_width(1.4);
        ctx.begin_path();
        for i in 0..=30 {
            let [x, y, h] = sm_grin_at(-SM_GRIN.amax + 2.0 * SM_GRIN.amax * i as f64 / 30.0);
            let yy = py(y + (if i % 2 != 0 { -h } else { h }) * 0.92);
            if i != 0 {
                ctx.line_to(px(x), yy);
            } else {
                ctx.move_to(px(x), yy);
            }
        }
        ctx.stroke();
        ctx.set_global_composite_operation("source-over");
    })
}
/// near it, only a faint cool halo (the teeth and eyes are real geometry)
fn make_smiler_halo_tex() -> Tex {
    canvas_tex_opt(128, 128, false, |ctx, _w, _h| {
        ctx.clear_rect(0, 0, 128, 128);
        let g = ctx.create_radial_gradient(64, 70, 2, 64, 70, 60);
        g.add_color_stop(0, "rgba(238,242,255,0.2)");
        g.add_color_stop(0.5, "rgba(238,242,255,0.05)");
        g.add_color_stop(1, "rgba(238,242,255,0)");
        ctx.set_fill_style(&g);
        ctx.fill_rect(0, 0, 128, 128);
    })
}
fn make_hound_face_tex() -> Tex {
    canvas_tex_opt(128, 128, false, |ctx, _w, _h| {
        ctx.clear_rect(0, 0, 128, 128);
        // two white points of eyeshine, 0.06 m apart on a 0.16 m plane, with
        // a wide cold bloom so they read through the hair at a few metres
        ctx.set_shadow_color("#e9f1e4");
        for (blur, r) in [(30.0, 9.0), (14.0, 8.0)] {
            ctx.set_shadow_blur(blur);
            ctx.set_fill_style("#f4faf0");
            ctx.begin_path();
            ctx.ellipse(40, 64, r, r - 1.0, 0, 0, 7);
            ctx.fill();
            ctx.begin_path();
            ctx.ellipse(88, 64, r, r - 1.0, 0, 0, 7);
            ctx.fill();
        }
    })
}

/// The impression of a face under the Watcher's sheet: the cloth sunk into
/// two eye hollows and a mouth stretched open, a lit ridge over the brow and
/// the nose, all soft-edged so it reads as cloth over something.
fn make_watcher_face_tex() -> Tex {
    canvas_tex_opt(128, 160, false, |ctx, _w, _h| {
        ctx.clear_rect(0, 0, 128, 160);
        let hollow = |ctx: &mut Canvas, x: f64, y: f64, rx: f64, ry: f64, a: f64| {
            let g = ctx.create_radial_gradient(x, y, 0, x, y, rx.max(ry));
            g.add_color_stop(0, &format!("rgba(18,15,12,{a})"));
            g.add_color_stop(0.55, &format!("rgba(30,26,22,{})", a * 0.55));
            g.add_color_stop(1, "rgba(40,36,30,0)");
            ctx.set_fill_style(&g);
            ctx.save();
            ctx.translate(x, y);
            ctx.scale(1.0, ry / rx.max(ry));
            ctx.translate(-x, -y);
            ctx.begin_path();
            ctx.arc(x, y, rx.max(ry), 0, 7);
            ctx.fill();
            ctx.restore();
        };
        // brow and nose ridge: the cloth pulled tight and catching the light
        let g = ctx.create_radial_gradient(64, 58, 4, 64, 58, 52);
        g.add_color_stop(0, "rgba(235,228,214,0.22)");
        g.add_color_stop(1, "rgba(235,228,214,0)");
        ctx.set_fill_style(&g);
        ctx.fill_rect(0, 0, 128, 160);
        hollow(ctx, 42.0, 66.0, 16.0, 13.0, 0.62);
        hollow(ctx, 86.0, 66.0, 16.0, 13.0, 0.62);
        // the mouth, too wide and pulled down, the cloth sucked into it
        hollow(ctx, 64.0, 118.0, 22.0, 30.0, 0.78);
        hollow(ctx, 64.0, 124.0, 10.0, 18.0, 0.55);
    })
}

// ------------------------------------------------------------ shared geometry
/// `sm_geo` — the Smiler's parts (joint at the origin, the bone down -Y, +Z forward)
pub struct SmGeo {
    /// `sm_body`: skinned, when built (tools/blender/skinned.py)
    pub body: Option<Geo>,
    pub torso: Geo,
    pub thigh: Geo,
    pub shin: Geo,
    pub meta: Geo,
    pub toes: Geo,
    pub up_arm: Geo,
    pub fore: Geo,
    pub hand: Geo,
    pub neck: Geo,
    pub head: Geo,
    pub jaw: Geo,
    pub teeth_up: Option<Geo>,
    pub teeth_lo: Option<Geo>,
    pub maw: Option<Geo>,
    /// deep in the lids (tools/blender/creatures.py SMH_EYE_*)
    pub eye: Geo,
}
/// `hd_geo` — the Hound's parts
pub struct HdGeo {
    /// `hd_skin`: skinned, when built (tools/blender/skin_hound.py)
    pub skin: Option<Geo>,
    pub body: Geo,
    pub neck: Geo,
    pub head: Geo,
    pub jaw: Geo,
    pub teeth_up: Option<Geo>,
    pub teeth_lo: Option<Geo>,
    pub ear: Option<Geo>,
    pub hair: Option<Geo>,
    pub up_arm: Geo,
    pub fore: Geo,
    pub hand: Geo,
    pub thigh: Geo,
    pub shin: Geo,
    pub foot: Geo,
    pub eye: Geo,
}
/// `wt_geo` — the Watcher's one sheet and its morph targets. `morph_color` are
/// the colour deltas the engine has no morph attribute for (blended on the CPU
/// per Watcher, see `apply_watcher_pose`); `base_color` the sheet's own colours.
pub struct WtGeo {
    pub body: Geo,
    pub morphs: usize,
    pub morph_color: Rc<Vec<Vec<f32>>>,
    pub base_color: Rc<Vec<f32>>,
}

/// Everything the entities build once: the materials, the billboard textures
/// and the shared geometry.
pub struct EntAssets {
    pub smiler_skin: Mat,
    pub smiler_tooth: Mat,
    pub smiler_maw: Mat,
    pub hound_skin: Mat,
    pub hound_hair: Mat,
    pub hound_tooth: Mat,
    pub hound_eye: Mat,
    pub watcher_cloth_tex: Tex,
    pub watcher_sheet: Mat,
    pub sm: SmGeo,
    pub hd: HdGeo,
    pub wt: WtGeo,
    pub smiler_face_tex: Tex,
    pub smiler_halo_tex: Tex,
    pub hound_face_tex: Tex,
    pub watcher_face_tex: Tex,
}

/// The generated mesh, or a crude primitive stand-in
fn cpart(name: &str, fb: impl FnOnce() -> Geo) -> Geo {
    creature_geometry(name).unwrap_or_else(fb)
}
fn limb_fb(r: f64, len: f64) -> Geo {
    let g = capsule_geo!(r, len, 3, 6);
    g.translate(0.0, -len / 2.0, 0.0);
    g
}

fn build_assets() -> EntAssets {
    // near-black and wet; sharp wet highlights with a faint oily violet
    let smiler_skin = with_skin_detail(phong!({ color: 0xffffff, vertexColors: true, specular: 0x4a4654, shininess: 70 }), 3.1, 1.05);
    let smiler_tooth = basic_mat!({ color: 0xffffff, vertexColors: true }); // unlit white, greyer at the root
    let smiler_maw = basic_mat!({ color: 0x000000 }); // the mouth is black, never red
                                                      // grey-pallid human skin, matte and a little waxy
    let hound_skin = with_skin_detail(phong!({ color: 0xffffff, vertexColors: true, specular: 0x1e1c1a, shininess: 22 }), 3.4, 1.2);
    // wet, matted hair: a dull broad sheen, not the tight highlight of plastic
    let hound_hair = phong!({ color: 0x0a0909, specular: 0x1c1b1a, shininess: 18 });
    let hound_tooth = lambert!({ color: 0xffffff, vertexColors: true });
    let hound_eye = basic_mat!({ color: 0xe9f1e4 });
    // the aged dust sheet, grey with handling and damp, a faint floor of light, its light rolling
    // off logarithmically past 0.5 (Patch::WatcherSheet = skin detail + rolloff)
    let watcher_cloth_tex = make_watcher_cloth_tex();
    let watcher_sheet = with_skin_detail(lambert!({ color: 0xc8c0b2, vertexColors: true, emissive: 0x100f0c }), 3.1, 0.85);
    {
        let mut m = watcher_sheet.m();
        m.bump_map = Some(watcher_cloth_tex.clone());
        m.patch = Patch::WatcherSheet { map: watcher_cloth_tex.clone(), scale: 3.1, strength: 0.85 };
    }
    let sm = SmGeo {
        body: creature_geometry("smBody"),
        torso: cpart("smTorso", || {
            let g = capsule_geo!(0.12, 0.6, 4, 8);
            g.translate(0, 0.35, 0);
            g
        }),
        thigh: cpart("smThigh", || limb_fb(0.04, 0.56)),
        shin: cpart("smShin", || limb_fb(0.025, 0.56)),
        meta: cpart("smMeta", || limb_fb(0.02, 0.3)),
        toes: cpart("smToes", || {
            let g = box_geo!(0.07, 0.03, 0.12);
            g.translate(0, -0.01, 0.05);
            g
        }),
        up_arm: cpart("smUpArm", || limb_fb(0.03, 0.48)),
        fore: cpart("smFore", || limb_fb(0.022, 0.46)),
        hand: cpart("smHand", || limb_fb(0.03, 0.26)),
        neck: cpart("smNeck", || {
            let g = limb_fb(0.032, 0.2);
            g.translate(0, 0.2, 0);
            g
        }),
        head: cpart("smHead", || sphere_geo!(0.13, 14, 10)),
        jaw: cpart("smJaw", || {
            let g = box_geo!(0.2, 0.05, 0.16);
            g.translate(0, -0.08, 0.03);
            g
        }),
        teeth_up: creature_geometry("smTeethUp"),
        teeth_lo: creature_geometry("smTeethLo"),
        maw: creature_geometry("smMaw"),
        eye: sphere_geo!(0.0155, 12, 9),
    };
    let hd = HdGeo {
        skin: creature_geometry("hdSkin"),
        body: cpart("hdBody", || {
            let g = capsule_geo!(0.14, 0.5, 4, 8);
            g.rotate_x(PI / 2.0);
            g
        }),
        neck: cpart("hdNeck", || {
            let g = capsule_geo!(0.034, 0.18, 3, 6);
            g.rotate_x(PI / 2.0).translate(0, 0, 0.08);
            g
        }),
        head: cpart("hdHead", || {
            let g = sphere_geo!(0.09, 12, 8);
            g.translate(0, 0.05, 0.1);
            g
        }),
        jaw: cpart("hdJaw", || {
            let g = box_geo!(0.1, 0.03, 0.18);
            g.translate(0, -0.03, 0.17);
            g
        }),
        teeth_up: creature_geometry("hdTeethUp"),
        teeth_lo: creature_geometry("hdTeethLo"),
        ear: creature_geometry("hdEar"),
        hair: creature_geometry("hdHair"),
        up_arm: cpart("hdUpArm", || limb_fb(0.03, 0.31)),
        fore: cpart("hdFore", || limb_fb(0.022, 0.27)),
        hand: cpart("hdHand", || {
            let g = box_geo!(0.08, 0.025, 0.18);
            g.translate(0, -0.025, 0.08);
            g
        }),
        thigh: cpart("hdThigh", || limb_fb(0.045, 0.44)),
        shin: cpart("hdShin", || limb_fb(0.028, 0.42)),
        foot: cpart("hdFoot", || {
            let g = box_geo!(0.08, 0.04, 0.2);
            g.translate(0, -0.1, 0.05);
            g
        }),
        eye: sphere_geo!(0.011, 8, 6),
    };
    // the Watcher is one sheet; the morphs `wtPose1`..`wtPose4` and `wtBow` are relative targets
    let w = watcher_geometry();
    let base_color = Rc::new(w.body.r().color.clone());
    let wt = WtGeo { body: w.body, morphs: w.morphs, morph_color: Rc::new(w.morph_color), base_color };
    EntAssets {
        smiler_skin,
        smiler_tooth,
        smiler_maw,
        hound_skin,
        hound_hair,
        hound_tooth,
        hound_eye,
        watcher_cloth_tex,
        watcher_sheet,
        sm,
        hd,
        wt,
        smiler_face_tex: make_smiler_face_tex(),
        smiler_halo_tex: make_smiler_halo_tex(),
        hound_face_tex: make_hound_face_tex(),
        watcher_face_tex: make_watcher_face_tex(),
    }
}

thread_local! { static ASSETS: std::cell::OnceCell<Rc<EntAssets>> = const { std::cell::OnceCell::new() }; }
/// The entities' shared materials, textures and geometry (built once, on the
/// native RNG, on first use).
pub fn ent_assets() -> Rc<EntAssets> {
    ASSETS.with(|c| c.get_or_init(|| Rc::new(with_native(build_assets))).clone())
}

// ------------------------------------------------------------ rigs
/// `SM_LEG`: the hip height at rest and the bone lengths of the Blender
/// parts; the rest angles put the ball of the foot on the floor.
pub struct SmLegSpec {
    pub hip_y: f64,
    pub hip_x: f64,
    pub thigh: f64,
    pub shin: f64,
    pub meta: f64,
    pub ball: f64,
    pub rest_hip: f64,
    pub rest_knee: f64,
    pub rest_ankle: f64,
}
pub const SM_LEG: SmLegSpec = SmLegSpec {
    hip_y: 1.226,
    hip_x: 0.108,
    thigh: 0.56,
    shin: 0.56,
    meta: 0.30,
    ball: 0.025,
    rest_hip: -0.49,
    rest_knee: 1.14,
    rest_ankle: -1.175,
};
/// (torso frame: it is built hunched)
pub const SM_SHOULDER: [f64; 3] = [0.168, 0.645, 0.04];
/// The Smiler's abdomen and chest joints (pelvis frame)
pub const SM_SPINE_ABDOMEN: [f64; 3] = [0.0, 0.27, 0.0];
pub const SM_SPINE_CHEST: [f64; 3] = [0.0, 0.23, 0.012];

/// The Smiler hand's finger joints (palm frame: wrist at the origin,
/// hanging -Y, palm +Z, thumb +X): four long fingers of three bones, curled a
/// little at rest, and the thumb of two.
pub fn sm_fingers() -> &'static Vec<Vec<[f64; 3]>> {
    static F: OnceLock<Vec<Vec<[f64; 3]>>> = OnceLock::new();
    F.get_or_init(|| {
        let curl = [0.32, 0.5, 0.45];
        let ll = [0.08, 0.07, 0.056];
        let mut out = vec![];
        for (k, &x) in [-0.03, -0.01, 0.01, 0.03].iter().enumerate() {
            let sc = [0.9, 1.0, 0.97, 0.84][k];
            let mut p = [x, -0.112, 0.0];
            let mut ang = 0.0;
            let mut pts = vec![p];
            for j in 0..3 {
                ang += curl[j];
                let d = [x * 0.35, -f64::cos(ang), f64::sin(ang)];
                let l = hypot3(d[0], d[1], d[2]);
                p = [p[0] + d[0] / l * ll[j] * sc, p[1] + d[1] / l * ll[j] * sc, p[2] + d[2] / l * ll[j] * sc];
                pts.push(p);
            }
            out.push(pts);
        }
        out.push(vec![[0.04, -0.055, 0.016], [0.05, -0.1, 0.034], [0.052, -0.142, 0.054]]); // the thumb: two bones
        out
    })
}

/// `HD_RIG`: joints in the root frame (the spine joint, 0.70 m up) and the
/// bone lengths of the Blender parts.
pub struct HdRig {
    pub spine_y: f64,
    pub shoulder: [f64; 3],
    pub hip: [f64; 3],
    pub up_arm: f64,
    pub fore: f64,
    pub wrist_h: f64,
    pub thigh: f64,
    pub shin: f64,
    pub ball: [f64; 2],
    pub hand_z: f64,
    pub foot_z: f64,
    pub hand_x: f64,
    pub foot_x: f64,
    pub neck_pitch: f64,
    pub neck_len: f64,
    pub pelvis: [f64; 3],
    pub lumbar: [f64; 3],
    pub thorax: [f64; 3],
    pub ribs: [f64; 3],
    pub scapula: [f64; 3],
    pub neck: [f64; 3],
    pub knuckle: [f64; 3],
}
pub const HD_RIG: HdRig = HdRig {
    spine_y: 0.70,
    shoulder: [0.15, -0.075, 0.30],
    hip: [0.12, 0.12, -0.25],
    up_arm: 0.32,
    fore: 0.29,
    wrist_h: 0.035,
    thigh: 0.44,
    shin: 0.42,
    ball: [0.128, 0.10],
    hand_z: 0.3,
    foot_z: -0.3,
    hand_x: 0.17,
    foot_x: 0.135,
    neck_pitch: 1.05,
    neck_len: 0.21,
    pelvis: [0.0, 0.12, -0.24],
    lumbar: [0.0, 0.1, -0.08],
    thorax: [0.0, 0.062, 0.08],
    ribs: [0.0, -0.055, 0.11],
    scapula: [0.078, 0.052, 0.19],
    neck: [0.0, -0.06, 0.34],
    knuckle: [0.0, -0.022, 0.09],
};
/// The Hound's digits, joint by joint from the knuckle to the tip.
pub struct HdDigits {
    pub hand: Vec<Vec<[f64; 3]>>,
    pub foot: Vec<Vec<[f64; 3]>>,
    pub splay: [f64; 5],
}
pub fn hd_digits() -> &'static HdDigits {
    static D: OnceLock<HdDigits> = OnceLock::new();
    D.get_or_init(|| {
        let fan = |x: f64, z: f64, l: f64, a: f64, drop: [f64; 3]| {
            let mut pts = vec![[x, -0.022, z]];
            for (j, f) in [0.46, 0.31, 0.23].iter().enumerate() {
                let p = pts[j];
                pts.push([p[0] + a.sin() * l * f, p[1] - drop[j], p[2] + a.cos() * l * f]);
            }
            pts
        };
        let hand = vec![
            fan(-0.033, 0.078, 0.068, -0.3, [0.003, 0.003, 0.002]), // little
            fan(-0.012, 0.09, 0.082, -0.1, [0.003, 0.003, 0.002]),  // ring
            fan(0.009, 0.094, 0.09, 0.06, [0.003, 0.003, 0.002]),   // middle
            fan(0.03, 0.088, 0.082, 0.22, [0.003, 0.003, 0.002]),   // index
            vec![[0.028, -0.016, 0.02], [0.052, -0.02, 0.05], [0.066, -0.024, 0.078], [0.074, -0.027, 0.099]], // thumb
        ];
        let foot = [0.052, 0.046, 0.041, 0.036, 0.03]
            .iter()
            .enumerate()
            .map(|(k, &l)| {
                let x = -0.03 + k as f64 * 0.015;
                let p0 = [x, -0.112, 0.118];
                let p1 = [p0[0] + x * 0.08, p0[1] + 0.004, p0[2] + l * 0.55];
                vec![p0, p1, [p1[0], p1[1] - 0.013, p1[2] + l * 0.35]]
            })
            .collect();
        HdDigits { hand, foot, splay: [-1.0, -0.45, 0.1, 0.6, 0.9] } // how each finger fans out as it splays
    })
}

/// One of `WT_POSES` (WT_POSE_SPECS in tools/blender/creatures.py): head roll
/// and nod, which hand moves, the forward lean, and where the head's centre
/// ends up from the neck joint.
pub struct WtPose {
    pub tilt: f64,
    pub pitch: f64,
    pub arm: &'static str,
    pub lean: f64,
    pub head: [f64; 3],
}
pub const WT_POSES: [WtPose; 5] = [
    WtPose { tilt: 0.0, pitch: 0.21, arm: "down", lean: 0.0, head: [0.0, 0.147, 0.026] }, // upright, arms down
    WtPose { tilt: 0.35, pitch: 0.24, arm: "down", lean: 0.0, head: [-0.024, 0.142, 0.027] }, // the head tilted 20 degrees
    WtPose { tilt: 0.26, pitch: 0.27, arm: "chest", lean: 0.0, head: [-0.018, 0.142, 0.029] }, // a hand raised to the chest
    WtPose { tilt: 0.1, pitch: 0.2, arm: "mouth", lean: 0.0, head: [-0.007, 0.147, 0.025] }, // a hand over the mouth
    WtPose { tilt: 0.24, pitch: 0.3, arm: "mouth", lean: 0.17, head: [-0.017, 0.114, 0.226] }, // leaning forward, ten degrees
];
/// The bowed head (the `wtBow` morph), from the neck joint
pub const WT_BOW_HEAD: [f64; 3] = [0.0, 0.134, 0.04];

/// A Smiler leg
#[derive(Clone, Debug)]
pub struct SmLeg {
    pub hip: Obj,
    pub knee: Obj,
    pub ankle: Obj,
    pub toes: Obj,
    pub side: f64,
}
/// A Smiler arm
#[derive(Clone, Debug)]
pub struct SmArm {
    pub shoulder: Obj,
    pub elbow: Obj,
    pub wrist: Obj,
    pub palm: Obj,
    pub fingers: Vec<Vec<Obj>>,
    pub side: f64,
    pub y0: f64,
}
/// The Smiler's typed rig
#[derive(Clone, Debug)]
pub struct SmilerRig {
    pub legs: Vec<SmLeg>,
    pub arms: Vec<SmArm>,
    pub torso: Obj,
    pub abdomen: Obj,
    pub chest: Obj,
    pub neck: Obj,
    pub head: Obj,
    pub jaw: Obj,
    pub eye_mat: Mat,
    pub skin: Option<Obj>,
    /// the billboard materials: the halo close up, the drawn face far off
    pub halo: Mat,
    pub far: Mat,
}
/// A Hound limb
#[derive(Clone, Debug)]
pub struct HdLeg {
    pub hip: Obj,
    pub knee: Obj,
    pub foot: Obj,
    pub scap: Option<Obj>,
    pub digits: Vec<Vec<Obj>>,
    pub front: bool,
    pub side: f64,
    pub lame: bool,
    pub base_y: f64,
    pub base_z: f64,
    pub off: [f64; 3],
}
/// The Hound's typed rig
#[derive(Clone, Debug)]
pub struct HoundRig {
    pub legs: Vec<HdLeg>,
    pub spine: Obj,
    pub pelvis: Obj,
    pub lumbar: Obj,
    pub thorax: Obj,
    pub ribs: Obj,
    pub neck: Obj,
    pub head: Obj,
    pub jaw: Obj,
    pub hair: Obj,
    /// the tail's root, mid and tip joints
    pub tail: Option<[Obj; 3]>,
    pub ears: Vec<Obj>,
    pub skin: Option<Obj>,
    pub far: Mat,
}
/// The Watcher's typed rig
#[derive(Clone, Debug)]
pub struct WatcherRig {
    pub torso: Obj,
    pub sheet: Obj,
    pub neck: Obj,
    pub head: Obj,
    /// the morph colour deltas and base colours (shared), blended into this
    /// Watcher's own copy of the sheet when its influences change
    pub morph_color: Rc<Vec<Vec<f32>>>,
    pub base_color: Rc<Vec<f32>>,
    pub blended: RefCell<Vec<f64>>,
}
/// A typed rig, one kind per creature.
#[derive(Clone, Debug)]
pub enum Rig {
    Smiler(SmilerRig),
    Hound(HoundRig),
    Watcher(WatcherRig),
}

/// The tic spring of `entity_tics`
#[derive(Clone, Copy, Debug, Default)]
pub struct Tic {
    pub p: [f64; 3],
    pub v: [f64; 3],
    pub g: [f64; 3],
    pub next: f64,
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
    /// the typed rig
    pub rig: Option<Rc<Rig>>,
    /// the aim field (compared by identity)
    pub aim_field: Option<Rc<Vec<i32>>>,
    /// the tic spring
    pub tic: Option<Tic>,
    /// the voice
    pub voice: Option<EntVoice>,
    /// a creature of the redesign: its body and brain (`creature`)
    pub creature: Option<Box<creature::Creature>>,
}

/// Store a bone's tail in its user data (where its tip is, for the Blender tools)
fn set_tail(o: &Obj, t: [f64; 3]) {
    let mut u = o.user();
    u.set("tailX", t[0]);
    u.set("tailY", t[1]);
    u.set("tailZ", t[2]);
}
/// Store a bone's main child in its user data (for the Blender tools)
fn set_main(o: &Obj, m: &str) {
    o.user().s.insert("main".into(), m.to_string());
}
/// Every named bone under `root`, by name (the rest of `parts` is added by the maker).
fn bones_by_name(root: &Obj) -> HashMap<String, Obj> {
    let mut m = HashMap::new();
    root.traverse(&mut |o| {
        if o.is_bone() {
            let n = o.name();
            if !n.is_empty() {
                m.insert(n, o.clone());
            }
        }
    });
    m
}
/// A joint from its parent (root frame)
fn rel(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn jnt(geo: Option<Geo>, mat: &Mat, parent: &Obj, p: [f64; 3], name: &str) -> Obj {
    joint(geo, mat, Some(parent), p[0], p[1], p[2], name)
}

impl Game {
    /// start-up work (creature materials, textures, shared geometry)
    pub fn init_entities(&mut self) {
        let _ = ent_assets();
        self.entities.clear();
        self.player_field = None;
        self.player_field_t = 0.0;
        self.heart_t = 0.0;
    }

    /// The typed rig of entity `ei`, if it has one.
    pub fn entity_rig(&self, ei: usize) -> Option<Rc<Rig>> {
        self.entities.get(ei).and_then(|e| self.ent_ext.get(&e.g.id())).and_then(|x| x.rig.clone())
    }

    /// Tall, starved, black, and it stoops; digitigrade
    /// legs, arms past the knees, a face that is everything.
    pub fn make_smiler(&mut self, x: f64, z: f64) -> EntityBuild {
        let a = ent_assets();
        let skin_m = &a.smiler_skin;
        let g = group();
        let hip_y = 1.226; // = SM_LEG.hip_y
        let body = a.sm.body.clone(); // the skinned body, when the Blender build has one
        let rigid = |geo: &Geo| if body.is_some() { None } else { Some(geo.clone()) };
        let mut legs = vec![];
        for s in [-1.0, 1.0] {
            let n = if s < 0.0 { "R" } else { "L" };
            let hip = joint(rigid(&a.sm.thigh), skin_m, Some(&g), s * SM_LEG.hip_x, hip_y, 0.0, &format!("thigh.{n}"));
            hip.rotation().x = SM_LEG.rest_hip;
            let knee = joint(rigid(&a.sm.shin), skin_m, Some(&hip), 0.0, -SM_LEG.thigh, 0.0, &format!("shin.{n}"));
            knee.rotation().x = SM_LEG.rest_knee;
            let ankle = joint(rigid(&a.sm.meta), skin_m, Some(&knee), 0.0, -SM_LEG.shin, 0.0, &format!("meta.{n}")); // the hock
            ankle.rotation().x = SM_LEG.rest_ankle;
            let toes = joint(rigid(&a.sm.toes), skin_m, Some(&ankle), 0.0, -SM_LEG.meta, 0.0, &format!("toes.{n}"));
            toes.rotation().x = -(SM_LEG.rest_hip + SM_LEG.rest_knee + SM_LEG.rest_ankle); // flat on the floor
            set_tail(&toes, [0.0, -0.012, 0.1]);
            legs.push(SmLeg { hip, knee, ankle, toes, side: s });
        }
        let torso = joint(rigid(&a.sm.torso), skin_m, Some(&g), 0.0, hip_y, 0.0, "pelvis");
        torso.rotation().x = 0.12;
        let abdomen = jnt(None, skin_m, &torso, SM_SPINE_ABDOMEN, "abdomen");
        let chest = jnt(None, skin_m, &abdomen, SM_SPINE_CHEST, "chest");
        set_main(&torso, "abdomen");
        set_main(&chest, "neck");
        let (cy, cz) = (SM_SPINE_ABDOMEN[1] + SM_SPINE_CHEST[1], SM_SPINE_ABDOMEN[2] + SM_SPINE_CHEST[2]);
        let mut arms = vec![];
        for s in [-1.0, 1.0] {
            let n = if s < 0.0 { "R" } else { "L" };
            let shoulder = joint(
                rigid(&a.sm.up_arm),
                skin_m,
                Some(&chest),
                s * SM_SHOULDER[0],
                SM_SHOULDER[1] - cy,
                SM_SHOULDER[2] - cz,
                &format!("upperarm.{n}"),
            );
            // elbows turned out; bound in an A-pose (the animation lets the arms hang)
            shoulder.set_rot(-0.12, -s * 0.5, s * 0.34);
            let elbow = joint(rigid(&a.sm.fore), skin_m, Some(&shoulder), 0.0, -0.48, 0.0, &format!("forearm.{n}"));
            elbow.rotation().x = -0.25;
            let wrist = joint(None, skin_m, Some(&elbow), 0.0, -0.46, 0.0, &format!("wrist.{n}"));
            // the palm turns the hand to the thigh, thumb forward; mirrored on the right
            let palm = joint(rigid(&a.sm.hand), skin_m, Some(&wrist), 0.0, 0.0, 0.0, &format!("palm.{n}"));
            palm.scale().x = s;
            palm.rotation().y = -s * PI / 2.0;
            set_tail(&palm, [0.0, -0.1, 0.0]);
            let fingers: Vec<Vec<Obj>> = sm_fingers()
                .iter()
                .enumerate()
                .map(|(k, pts)| {
                    let mut chain = vec![];
                    let mut parent = palm.clone();
                    let mut prev = [0.0; 3];
                    for j in 0..pts.len() - 1 {
                        let b = jnt(None, skin_m, &parent, rel(pts[j], prev), &format!("finger{k}{j}.{n}"));
                        chain.push(b.clone());
                        prev = pts[j];
                        parent = b;
                    }
                    set_tail(&parent, rel(pts[pts.len() - 1], pts[pts.len() - 2]));
                    chain
                })
                .collect();
            set_main(&palm, &format!("finger10.{n}"));
            let y0 = shoulder.pos().y;
            arms.push(SmArm { shoulder, elbow, wrist, palm, fingers, side: s, y0 });
        }
        let neck = joint(rigid(&a.sm.neck), skin_m, Some(&chest), 0.0, 0.715 - cy, 0.05 - cz, "neck");
        neck.rotation().x = 0.55; // the head juts forward
        set_main(&neck, "head");
        let head = joint(None, skin_m, Some(&neck), 0.0, 0.3, 0.0, "head");
        head.rotation().x = -0.67; // level again
        set_tail(&head, [0.0, 0.16, 0.02]);
        // the eyes first (tests find them as the head's first small spheres)
        let eye_mat = smiler_eye_material();
        for s in [-1.0, 1.0] {
            let eye = mesh(&a.sm.eye, &eye_mat);
            eye.set_pos(s * 0.047, 0.035, 0.062);
            eye.scale().set(1.35, 1.35, 1.35);
            head.add(&eye);
        }
        head.add(&mesh(&a.sm.head, skin_m));
        if let Some(m) = &a.sm.maw {
            head.add(&mesh(m, &a.smiler_maw));
        }
        if let Some(t) = &a.sm.teeth_up {
            head.add(&mesh(t, &a.smiler_tooth));
        }
        // the jaw hinges far back, where ears would be (SMH_HINGE)
        let jaw = joint(None, skin_m, Some(&head), 0.0, -0.012, -0.052, "jaw");
        set_tail(&jaw, [0.0, -0.06, 0.13]);
        let jaw_mesh = mesh(&a.sm.jaw, skin_m);
        jaw_mesh.set_pos(0.0, 0.012, 0.052);
        jaw.add(&jaw_mesh);
        if let Some(t) = &a.sm.teeth_lo {
            let lower = mesh(t, &a.smiler_tooth);
            lower.set_pos(0.0, 0.012, 0.052);
            jaw.add(&lower);
        }
        let skin = match &body {
            Some(b) => skin_body(&g, b, skin_m),
            None => None,
        };

        g.set_pos(self.wx(x), 0.0, self.wz(z));
        g.scale().set(1.08, 1.08, 1.08); // a head taller than a tall man
        if let Some(sk) = &skin {
            sync_skin_bind(sk);
        }
        // the billboard: a faint cool halo close up; past ~8 m, the drawn face
        let face_geo = plane_geo!(0.6, 0.6);
        let halo = basic_mat!({ map: &a.smiler_halo_tex, transparent: true, depthWrite: false });
        let face = mesh(&face_geo, &halo);
        let far = basic_mat!({ map: &a.smiler_face_tex, transparent: true, depthWrite: false, fog: false, opacity: 0 });
        face.add(&mesh(&face_geo, &far));
        self.scene.add(&face);
        self.world.add(&g);

        let mut parts = bones_by_name(&g);
        for (k, o) in [("torso", &torso), ("abdomen", &abdomen), ("chest", &chest), ("neck", &neck), ("head", &head), ("jaw", &jaw)] {
            parts.insert(k.into(), o.clone());
        }
        if let Some(sk) = &skin {
            parts.insert("skin".into(), sk.clone());
        }
        let rig = Rig::Smiler(SmilerRig { legs, arms, torso, abdomen, chest, neck, head, jaw, eye_mat, skin, halo, far });
        self.ent_ext.entry(g.id()).or_default().rig = Some(Rc::new(rig));
        EntityBuild { g, face: Some(face), face_y: 2.2, parts }
    }

    /// A starved person on all fours, the rump high, the
    /// head hanging under a curtain of wet black hair; the right hind leg lame.
    pub fn make_hound(&mut self, x: f64, z: f64) -> EntityBuild {
        let a = ent_assets();
        let skin_m = &a.hound_skin;
        let hair_m = &a.hound_hair;
        let g = group();
        let body = a.hd.skin.clone(); // the skinned body, when the Blender build has one
        let rigid = |geo: &Geo| if body.is_some() { None } else { Some(geo.clone()) };
        // the spine: a root at the old one-piece frame, then pelvis → lumbar → thorax
        let spine = joint(rigid(&a.hd.body), skin_m, Some(&g), 0.0, 0.0, 0.0, "root");
        spine.position().y = 0.70;
        let pelvis = jnt(None, skin_m, &spine, HD_RIG.pelvis, "pelvis");
        let lumbar = jnt(None, skin_m, &pelvis, rel(HD_RIG.lumbar, HD_RIG.pelvis), "lumbar");
        let thorax = jnt(None, skin_m, &lumbar, rel(HD_RIG.thorax, HD_RIG.lumbar), "thorax");
        let ribs = jnt(None, skin_m, &thorax, rel(HD_RIG.ribs, HD_RIG.thorax), "ribs"); // it breathes (scale)
        set_main(&pelvis, "lumbar");
        set_main(&thorax, "neck");
        set_tail(&spine, [0.0, 0.0, 0.1]);
        set_tail(&ribs, [0.0, -0.015, 0.15]);
        // no hair down the back: a person's bare, starved back, the spine and
        // the ribs showing (the tail bones stay, bare, for the gait)
        let tail_root = jnt(None, hair_m, &pelvis, rel([0.0, 0.16, -0.41], HD_RIG.pelvis), "tail0");
        let tail_mid = joint(None, hair_m, Some(&tail_root), 0.0, -0.13, -0.035, "tail1");
        let tail_tip = joint(None, hair_m, Some(&tail_mid), 0.0, -0.12, -0.035, "tail2");
        tail_root.rotation().x = 0.35;
        tail_mid.set_scalar(0.85);
        tail_tip.set_scalar(0.8);

        let neck = jnt(rigid(&a.hd.neck), skin_m, &thorax, rel(HD_RIG.neck, HD_RIG.thorax), "neck");
        neck.rotation().x = HD_RIG.neck_pitch; // hangs forward and down
        set_main(&neck, "head");
        let head = joint(None, skin_m, Some(&neck), 0.0, 0.0, HD_RIG.neck_len, "head");
        head.rotation().x = -HD_RIG.neck_pitch; // the head frame stays level; the skull is modelled hanging
        set_tail(&head, [0.0, 0.02, 0.2]);
        // the eyes first (tests find them as the head's first small spheres): small white eyeshine
        for s in [-1.0, 1.0] {
            let eye = mesh(&a.hd.eye, &a.hound_eye);
            eye.set_pos(s * 0.03, 0.055, 0.1495);
            head.add(&eye);
        }
        head.add(&mesh(&a.hd.head, skin_m));
        if let Some(t) = &a.hd.teeth_up {
            head.add(&mesh(t, &a.hound_tooth));
        }
        let jaw = joint(None, skin_m, Some(&head), 0.0, 0.0332, 0.0489, "jaw"); // the jaw splits back to under the ears
        set_tail(&jaw, [0.0, -0.06, 0.15]);
        let jaw_mesh = mesh(&a.hd.jaw, skin_m);
        jaw_mesh.set_pos(0.0, -0.0332, -0.0489);
        jaw.add(&jaw_mesh);
        if let Some(t) = &a.hd.teeth_lo {
            let lower = mesh(t, &a.hound_tooth);
            lower.set_pos_v(&jaw_mesh.pos());
            jaw.add(&lower);
        }
        let mut ears = vec![];
        for s in [-1.0, 1.0] {
            // human ears, pushing through the hair; they turn to sounds
            let ear = joint(a.hd.ear.clone(), skin_m, Some(&head), s * 0.077, 0.079, 0.068, if s < 0.0 { "ear.R" } else { "ear.L" });
            ear.scale().x = s;
            ears.push(ear);
        }
        // the hair hangs from the crown on a spring (`animate_entity` lags it)
        let hair = joint(None, skin_m, Some(&head), 0.0, 0.11, 0.07, "hair");
        if let Some(h) = &a.hd.hair {
            let hm = mesh(h, hair_m);
            hm.set_pos(0.0, -0.11, -0.07);
            hair.add(&hm);
        }

        // arms and legs: three joints each and the digits, solved onto their
        // contact points; the arms hang from shoulder blades
        let dg = hd_digits();
        let mut legs = vec![];
        for (front, s) in [(true, -1.0), (true, 1.0), (false, -1.0), (false, 1.0)] {
            let n = if s < 0.0 { "R" } else { "L" };
            let jj = if front { HD_RIG.shoulder } else { HD_RIG.hip };
            let at = [s * jj[0], jj[1], jj[2]];
            let mut parent = pelvis.clone();
            let mut base = rel(at, HD_RIG.pelvis);
            let mut scap = None;
            if front {
                let sc = [s * HD_RIG.scapula[0], HD_RIG.scapula[1], HD_RIG.scapula[2]];
                let sj = jnt(None, skin_m, &thorax, rel(sc, HD_RIG.thorax), &format!("scapula.{n}"));
                set_main(&sj, &format!("upperarm.{n}"));
                parent = sj.clone();
                base = rel(at, sc);
                scap = Some(sj);
            }
            let hip = jnt(
                rigid(if front { &a.hd.up_arm } else { &a.hd.thigh }),
                skin_m,
                &parent,
                base,
                &format!("{}{n}", if front { "upperarm." } else { "thigh." }),
            );
            hip.set_rotation_order(EulerOrder::ZYX);
            let knee = joint(
                rigid(if front { &a.hd.fore } else { &a.hd.shin }),
                skin_m,
                Some(&hip),
                0.0,
                -(if front { HD_RIG.up_arm } else { HD_RIG.thigh }),
                0.0,
                &format!("{}{n}", if front { "forearm." } else { "shin." }),
            );
            let foot = joint(
                rigid(if front { &a.hd.hand } else { &a.hd.foot }),
                skin_m,
                Some(&knee),
                0.0,
                -(if front { HD_RIG.fore } else { HD_RIG.shin }),
                0.0,
                &format!("{}{n}", if front { "hand." } else { "foot." }),
            );
            foot.scale().x = if front { -s } else { s }; // thumbs and big toes on the inside
            let pre = if front { "finger" } else { "toe" };
            let digits: Vec<Vec<Obj>> = (if front { &dg.hand } else { &dg.foot })
                .iter()
                .enumerate()
                .map(|(k, pts)| {
                    let mut chain = vec![];
                    let mut par = foot.clone();
                    let mut prev = [0.0; 3];
                    for j in 0..pts.len() - 1 {
                        let b = jnt(None, skin_m, &par, rel(pts[j], prev), &format!("{pre}{k}{j}.{n}"));
                        chain.push(b.clone());
                        prev = pts[j];
                        par = b;
                    }
                    set_tail(&par, rel(pts[pts.len() - 1], pts[pts.len() - 2]));
                    chain
                })
                .collect();
            set_main(&foot, &format!("{pre}20.{n}"));
            legs.push(HdLeg {
                hip,
                knee,
                foot,
                scap,
                digits,
                front,
                side: s,
                lame: !front && s < 0.0,
                base_y: base[1],
                base_z: base[2],
                off: if front { [0.0, HD_RIG.wrist_h, -HD_RIG.knuckle[2]] } else { [0.0, HD_RIG.ball[0], -HD_RIG.ball[1]] },
            });
        }
        let mut rig = HoundRig {
            legs,
            spine,
            pelvis,
            lumbar,
            thorax,
            ribs,
            neck,
            head,
            jaw,
            hair,
            tail: Some([tail_root, tail_mid, tail_tip]),
            ears,
            skin: None,
            far: basic_mat!(),
        };
        // the rest pose, which the body is modelled and bound in: standing,
        // every hand and foot planted, the hands turned out a little
        let (mp, mt) = crate::entities::anim::hound_spine(&rig);
        for l in &rig.legs {
            crate::entities::anim::hound_limb(
                &mp,
                &mt,
                l,
                l.side * (if l.front { HD_RIG.hand_x } else { HD_RIG.foot_x }),
                0.0,
                if l.front { HD_RIG.hand_z + HD_RIG.knuckle[2] } else { HD_RIG.foot_z },
                0.0,
                l.side * (if l.front { 0.12 } else { 0.05 }),
            );
        }
        rig.skin = match &body {
            Some(b) => skin_body(&g, b, skin_m),
            None => None,
        };

        g.set_pos(self.wx(x), 0.0, self.wz(z));
        if let Some(sk) = &rig.skin {
            sync_skin_bind(sk);
        }
        // far off, two small points of eyeshine, flickering behind the hair
        let face = node();
        let far = basic_mat!({ map: &a.hound_face_tex, transparent: true, depthWrite: false, fog: false, opacity: 0 });
        let shine = mesh(plane_geo!(0.16, 0.16), &far);
        shine.position().z = 0.16;
        face.add(&shine);
        rig.far = far;
        self.scene.add(&face);
        self.world.add(&g);

        let mut parts = bones_by_name(&g);
        for (k, o) in [
            ("spine", &rig.spine),
            ("pelvis", &rig.pelvis),
            ("lumbar", &rig.lumbar),
            ("thorax", &rig.thorax),
            ("ribs", &rig.ribs),
            ("neck", &rig.neck),
            ("head", &rig.head),
            ("jaw", &rig.jaw),
            ("hair", &rig.hair),
        ] {
            parts.insert(k.into(), o.clone());
        }
        if let Some(sk) = &rig.skin {
            parts.insert("skin".into(), sk.clone());
        }
        self.ent_ext.entry(g.id()).or_default().rig = Some(Rc::new(Rig::Hound(rig)));
        EntityBuild { g, face: Some(face), face_y: 0.52, parts }
    }

    /// A dust sheet over someone standing very still.
    /// Its poses are morph targets on the one sheet. Each Watcher gets its own
    /// copy of the sheet geometry when it has colour morphs, so its colours
    /// can follow its own influences (the engine has no colour morph attribute).
    pub fn make_watcher(&mut self, x: f64, z: f64) -> EntityBuild {
        let a = ent_assets();
        let g = group();
        let torso = group();
        torso.position().y = 0.0;
        g.add(&torso);
        let geo = if a.wt.morphs > 0 && !a.wt.morph_color.is_empty() { a.wt.body.duplicate() } else { a.wt.body.clone() };
        let sheet = mesh(&geo, &a.watcher_sheet);
        if a.wt.morphs > 0 {
            *sheet.morph_influences() = vec![0.0; a.wt.morphs];
        }
        torso.add(&sheet);
        // the head inside the sheet carries nothing: it is where the face anchor
        // sits, moved to each pose's head (`apply_watcher_pose`)
        let neck = group();
        neck.position().y = 2.02;
        neck.position().z = 0.03;
        torso.add(&neck);
        let head = group();
        head.position().y = 0.147;
        head.position().z = 0.026;
        neck.add(&head);
        head.rotation().x = WT_POSES[0].pitch; // bowed a little, looking at you

        g.set_pos(self.wx(x), 0.0, self.wz(z));
        // no eyes, no light: only the shape of a face pressed into the cloth
        // from inside, the mouth open, turned always to you
        let face = node();
        let imprint = mesh(plane_geo!(0.34, 0.42), &lambert!({ map: &a.watcher_face_tex, transparent: true, depthWrite: false }));
        imprint.position().y = -0.04;
        face.add(&imprint);
        self.scene.add(&face);
        self.world.add(&g);

        let mut parts = HashMap::new();
        for (k, o) in [("torso", &torso), ("sheet", &sheet), ("neck", &neck), ("head", &head)] {
            parts.insert(k.to_string(), o.clone());
        }
        let rig = Rig::Watcher(WatcherRig {
            torso,
            sheet,
            neck,
            head,
            morph_color: a.wt.morph_color.clone(),
            base_color: a.wt.base_color.clone(),
            blended: RefCell::new(vec![]),
        });
        self.ent_ext.entry(g.id()).or_default().rig = Some(Rc::new(rig));
        EntityBuild { g, face: Some(face), face_y: 2.18, parts }
    }

    /// (seeded: one draw per pick, and
    /// the type draws from level 2 / 3 on, then `add_entity`'s yaw)
    pub fn spawn_entities(&mut self, count: i32, dist_from_spawn: &[i32]) {
        for e in &self.entities {
            if let Some(f) = &e.face {
                self.scene.remove(f);
            }
        } // faces live in scene, not world
        self.entities.clear();
        self.ent_ext.clear();
        let mut open: Vec<(i32, i32)> = vec![];
        for z in 1..self.h - 1 {
            for x in 1..self.w - 1 {
                let i = self.t_idx(x, z);
                if self.tiles[i] == 0 && dist_from_spawn[i] as f64 > self.w as f64 * 0.45 {
                    open.push((x, z));
                }
            }
        }
        let mut i = 0;
        while i < count && !open.is_empty() {
            let (x, z) = open.remove((rnd() * open.len() as f64).floor() as usize);
            // pick type by level: watcher from level 2+, hound from level 3+
            let mut kind = "smiler";
            if self.depth() >= 2 && i > 0 && rnd() < 0.35 {
                kind = "watcher";
            }
            if self.depth() >= 3 && i > 0 && kind == "smiler" && rnd() < 0.5 {
                kind = "hound";
            }
            self.add_entity(kind, x, z, i);
            if kind == "watcher" {
                watcher::company(self, x, z);
            }
            i += 1;
        }
    }

    /// → the new entity's index
    pub fn add_entity(&mut self, kind: &str, x: i32, z: i32, i: i32) -> usize {
        let mut made = None;
        let built = if creature::on_new_brain(kind) {
            let (b, c) = self.make_creature(kind, x as f64, z as f64);
            made = Some(c);
            b
        } else if kind == "hound" {
            self.make_hound(x as f64, z as f64)
        } else if kind == "watcher" {
            self.make_watcher(x as f64, z as f64)
        } else {
            self.make_smiler(x as f64, z as f64)
        };
        // R770 eye-Y offset per entity type: distance from the head group's origin to
        // the 3D eye sphere centre (head-local Y), so `update_face` anchors the
        // billboard at the actual eye height
        let eye_y_offset = if kind == "hound" {
            0.055
        } else if kind == "watcher" {
            0.020
        } else {
            0.035
        };
        let mut e = Entity::new(kind, built.g.clone());
        e.face = built.face;
        e.face_y = built.face_y;
        e.eye_y_offset = eye_y_offset;
        e.parts = built.parts;
        e.yaw = rand1(PI * 2.0);
        e.state = "wander".into(); // wander | hunt | stalking
        e.speed = (if kind == "hound" {
            7.2
        } else if kind == "watcher" {
            3.2
        } else {
            5.5
        }) + (self.depth() as f64 * 0.12).min(0.9);
        e.w_field = None;
        e.w_target = None;
        e.w_timer = 0.0;
        e.lose_t = 0.0;
        e.freeze = 2.0 + i as f64 * 1.5;
        e.stare_t = 0.0;
        e.step_acc = 0.0;
        // watcher-specific: teleports closer when totally unseen for a while
        e.unseen_t = 0.0;
        if let Some(c) = made {
            self.ent_ext.entry(e.g.id()).or_default().creature = Some(Box::new(c));
        }
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
            } else if let Some(sk) = e.parts.get("skin") {
                sync_skin_bind(sk);
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
    pub(crate) fn e_ceil_at_world(&self, px: f64, pz: f64) -> f64 {
        match entity_nav() {
            Some(n) => n.ceil_at_world(self, px, pz),
            None => self.ceil_at_world(px, pz),
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
