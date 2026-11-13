//! Instancing: turns repeated static prop meshes into instanced meshes.
//!
//! The Blender prop parts are cached per size (`props`), so the same geometry
//! turns up again and again: every leg of every display bed, every carton on
//! every rack. After a level is built and settled, each static prop's opaque
//! meshes are bucketed by (10×10-tile chunk, geometry, material look), and
//! every bucket with more than one member becomes one instanced mesh. Chunks
//! keep frustum culling useful; the material "look" (type, colours, map,
//! flags) lets two placers' identical-looking materials share one draw. No
//! vertices are copied. Each instance keeps its own wear by use (how much
//! its prop is handled, where its floor is: `Node::use_wear`), so things
//! on two storeys, or one on the floor and one on a shelf, still share it.
//!
//! Only props flagged `user.static_prop` are touched (placers that registered
//! no animator and no interaction). Transparent, blended, depth-write-off and
//! non-mesh objects are left as they are.
use crate::base::numeric::fmt_num;
use engine::prelude::*;
use std::collections::{HashMap, HashSet};

/// Counts from one instancing pass: meshes considered, meshes instanced,
/// buckets made, and emptied groups removed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stats {
    pub meshes: usize,
    pub instanced: usize,
    pub buckets: usize,
    pub removed_groups: usize,
}

/// The material's type name (part of [`look_key`]).
pub fn material_type(k: MatKind) -> &'static str {
    match k {
        MatKind::Basic => "basic",
        MatKind::Lambert => "lambert",
        MatKind::Phong => "phong",
        MatKind::Standard => "standard",
        MatKind::Points => "points",
        MatKind::Sprite => "sprite",
        MatKind::LineBasic => "line",
        MatKind::Depth => "depth",
    }
}

/// The material "look": the key that decides whether two materials draw alike.
pub fn look_key(m: &Mat) -> String {
    let m = m.r();
    // which material kinds have a `color` / `emissive` / `emissive_intensity`
    let has_color = !matches!(m.kind, MatKind::Depth);
    let has_emissive = matches!(m.kind, MatKind::Lambert | MatKind::Phong | MatKind::Standard);
    let col = |has: bool, c: &Color| if has { c.get_hex_string() } else { "-".to_string() };
    let side = match m.side {
        Side::Front => 0,
        Side::Back => 1,
        Side::Double => 2,
    };
    [
        material_type(m.kind).to_string(),
        col(has_color, &m.color),
        col(has_emissive, &m.emissive),
        if has_emissive { fmt_num(m.emissive_intensity) } else { "1".to_string() },
        m.map.as_ref().map(|t| format!("t{}", t.id())).unwrap_or_else(|| "-".into()),
        side.to_string(),
        (if m.flat_shading { "1" } else { "0" }).to_string(),
        (if m.vertex_colors { "1" } else { "0" }).to_string(),
        (if m.fog { "1" } else { "0" }).to_string(),
        (if m.tone_mapped { "1" } else { "0" }).to_string(),
        if m.polygon_offset { format!("{}/{}", fmt_num(m.polygon_offset_factor), fmt_num(m.polygon_offset_units)) } else { "-".into() },
        fmt_num(if m.alpha_test == 0.0 || m.alpha_test.is_nan() { 0.0 } else { m.alpha_test }),
    ]
    .join("|")
}

/// Whether the object or its geometry carries morph targets (an object gets
/// morph influences when its geometry has morph attributes).
fn has_morphs(o: &Obj, g: &Geo) -> bool {
    !o.r().morph_influences.is_empty() || !g.r().morph_position.is_empty() || !g.r().morph_normal.is_empty()
}

struct Bucket {
    geo: Geo,
    mat: Mat,
    list: Vec<Obj>,
    /// what each member was part of (`placed_label`)
    labels: Vec<String>,
}

/// Instances the static props under `world`, the level's root group
/// (identity transform). `chunk_of(x, z)` is the chunk id of a world position
/// (the game uses `floor(tx(x) / 10) * 64 + floor(tz(z) / 10)`).
pub fn instancify_static(world: &Obj, chunk_of: &dyn Fn(f64, f64) -> i64) -> Stats {
    let mut st = Stats::default();
    let mut buckets: Vec<Bucket> = vec![];
    let mut by_key: HashMap<String, usize> = HashMap::new();
    world.update_matrix_world(true);
    for top in world.children() {
        {
            let u = &top.r().user;
            if !u.static_prop || u.no_instance {
                continue;
            }
        }
        if top.is_instanced() || top.is_light() {
            continue;
        }
        top.traverse(&mut |o: &Obj| {
            if !o.is_mesh() || o.is_instanced() || !o.visible() {
                return;
            }
            let Some(geo) = o.try_geo() else { return };
            let mats = o.mats();
            // an array of materials (the engine keeps every material list as a
            // Vec, so only a single material counts as `material` here)
            if mats.len() != 1 {
                return;
            }
            let m = mats[0].clone();
            {
                let mm = m.r();
                if mm.transparent || mm.blending != Blending::Normal || !mm.depth_write {
                    return;
                }
            }
            if has_morphs(o, &geo) {
                return;
            }
            st.meshes += 1;
            let mw = o.matrix_world();
            let (x, z) = (mw.e[12], mw.e[14]);
            let key = format!("{}#{}#{}", chunk_of(x, z), geo.id(), look_key(&m));
            let bi = match by_key.get(&key) {
                Some(&i) => i,
                None => {
                    buckets.push(Bucket { geo: geo.clone(), mat: m.clone(), list: vec![], labels: vec![] });
                    by_key.insert(key, buckets.len() - 1);
                    buckets.len() - 1
                }
            };
            buckets[bi].list.push(o.clone());
            buckets[bi].labels.push(placed_label(o));
        });
    }
    let mut touched: Vec<Obj> = vec![];
    let mut touched_set: HashSet<Obj> = HashSet::new();
    for b in &buckets {
        if b.list.len() < 2 {
            continue;
        }
        let im = instanced_mesh(&b.geo, &b.mat, b.list.len());
        let wear = b.list[0].use_wear();
        im.set_use_wear(wear);
        let own = b.list.iter().any(|o| o.use_wear() != wear);
        for (i, o) in b.list.iter().enumerate() {
            im.set_matrix_at(i, &o.matrix_world());
            if own {
                im.set_use_wear_at(i, o.use_wear());
            }
            if let Some(p) = o.parent() {
                if touched_set.insert(p.clone()) {
                    touched.push(p.clone());
                }
                p.remove(o);
            }
        }
        im.set_matrix_auto_update(false);
        {
            let mut u = im.user();
            u.static_prop = true;
            u.instanced = true;
            // what each instance was part of (the visual playtest names it so;
            // not a placer: the level's dump lists placed things only)
            u.s.insert("instance_labels".into(), b.labels.join("|"));
        }
        world.add(&im);
        st.instanced += b.list.len();
        st.buckets += 1;
    }
    // groups the meshes came out of that are now empty
    for g in touched {
        let mut p = Some(g);
        while let Some(cur) = p.clone() {
            if cur.ptr_eq(world) || cur.child_count() != 0 {
                break;
            }
            let Some(up) = cur.parent() else { break };
            up.remove(&cur);
            st.removed_groups += 1;
            p = Some(up);
        }
    }
    st
}

/// The placed thing a mesh is part of: its nearest ancestor's core kind
/// (`core_<kind>`) or placer; `level` for the architecture
fn placed_label(o: &Obj) -> String {
    let mut p = o.parent();
    while let Some(n) = p {
        {
            let u = n.user();
            if let Some(k) = u.s.get("kind") {
                return format!("core_{k}");
            }
            if let Some(pl) = &u.placer {
                return pl.clone();
            }
        }
        p = n.parent();
    }
    "level".into()
}
