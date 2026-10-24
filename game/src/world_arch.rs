//! backrooms.js "architecture meshes" (3618–3851), freezeStatic / warmUpGPU,
//! the level teardown (3852–3902) and the prop draw distance (3903–3948).
//!
//! Walls are instanced unit boxes scaled to each archgen solid; floors and
//! ceilings are instanced quads at each tile's own height; a vertical
//! "bulkhead" closes every step between two ceiling heights. Surfaces that
//! are not tile-sized use world-space UVs (`Patch::ArchUV`).
use crate::lighting::RF_MOSS;
use crate::prelude::*;
use crate::records::RegionInfo;
use std::collections::{HashMap, HashSet};

/// line 3732 `ARCH_CHUNK` — architecture is instanced per material and per 10×10-tile chunk
pub const ARCH_CHUNK: f64 = 10.0;
/// line 3832 `FIXTURE_HANG` — fixtures under a tall slab hang on rods at this height
pub const FIXTURE_HANG: f64 = 4.3;
/// line 3910
pub const PROP_FAR: f64 = 34.0;
pub const PROP_FAR_STORE: f64 = 26.0;

/// line 3641 `worldUVMaterial(map, scale, wallpaper = false)`
pub fn world_uv_material(map: Option<Tex>, scale: [f64; 2], wallpaper: bool) -> Mat {
    let m = lambert!({ map: map });
    m.m().patch = Patch::ArchUV { scale_x: scale[0], scale_y: scale[1], wallpaper };
    m
}

/// line 3707 `grimeOf(info)`
pub fn grime_of(info: &RegionInfo) -> i32 { if info.flags & RF_MOSS != 0 { 3 } else { info.grime.clamp(0, 3) } }

/// One item of an `addInstanced` list: its instance matrix.
pub type ArchWrite<'a, T> = &'a dyn Fn(&T) -> Mat4;

impl Game {
    /// lines 3028–3030: `wallMats` / `ceilMats` / `floorMats` (one per
    /// wallpaper / ceiling / carpet variant), built once
    pub fn ensure_surface_mats(&mut self) {
        if self.wall_mats.len() != self.tex.wallpaper_variants.len() {
            self.wall_mats = self.tex.wallpaper_variants.iter().map(|v| lambert!({ map: &v.tex })).collect();
        }
        if self.ceil_mats.len() != self.tex.ceiling_variants.len() {
            self.ceil_mats = self.tex.ceiling_variants.iter().map(|t| lambert!({ map: t, side: DoubleSide })).collect();
        }
        if self.floor_mats.len() != self.tex.carpet_variants.len() {
            self.floor_mats = self.tex.carpet_variants.iter().map(|t| lambert!({ map: t })).collect();
        }
    }

    /// line 3652 `archMat(key)` — key grammar: 'wp:<wallpaper variant>' |
    /// '<surface kind>:<grime>' for walls, 'floor:<kind>:<g>', 'ceil:<kind>:<g>'
    pub fn arch_mat(&mut self, key: &str) -> Mat {
        if let Some(m) = self.arch_mats.get(key) { return m.clone(); }
        let parts: Vec<&str> = key.split(':').collect();
        let (a, b, c) = (parts[0], parts.get(1).copied().unwrap_or(""), parts.get(2).copied().unwrap_or(""));
        let num = |s: &str| s.parse::<f64>().unwrap_or(f64::NAN);
        let m = if a == "wp" {
            let i = num(b) as usize;
            let tex = self.tex.wallpaper_variants.get(i).map(|v| v.tex.clone());
            world_uv_material(tex, [CELL, WALL_H], true)
        } else if a == "floor" {
            lambert!({ map: self.surfaces.floor(b, num(c)) })
        } else if a == "ceil" {
            lambert!({ map: self.surfaces.ceil(b, num(c)), side: DoubleSide })
        } else {
            world_uv_material(self.surfaces.wall(a, num(b)), self.surfaces.scale(a), false)
        };
        self.arch_mats.insert(key.to_string(), m.clone());
        m
    }

    /// line 3699 `wallRegionIdx(x, z)` — region whose room a wall tile faces
    pub fn wall_region_idx(&self, x: i32, z: i32) -> usize {
        for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let (nx, nz) = (x + dx, z + dz);
            if self.in_grid(nx, nz) && self.tiles[self.t_idx(nx, nz)] == 0 { return self.region_map[self.t_idx(nx, nz)] as usize; }
        }
        self.region_map[self.t_idx(x, z)] as usize
    }
    /// line 3709 `tileFullySolid(S, t)` — one box owned by tile t covers the whole tile from the floor
    pub fn tile_fully_solid(&self, s: &crate::archgen::Solids, t: usize) -> bool {
        let (x, z) = ((t as i32 % self.w) as f64, (t as i32 / self.w) as f64);
        let b = &s.boxes;
        for k in s.start[t]..s.start[t + 1] {
            let o = s.items[k as usize] as usize * 8;
            if (b[o + 4] as f64) <= 0.0 && (b[o] as f64) <= x + 1e-4 && (b[o + 1] as f64) <= z + 1e-4 && (b[o + 2] as f64) >= x + 1.0 - 1e-4
                && (b[o + 3] as f64) >= z + 1.0 - 1e-4 && (b[o + 5] as f64) >= self.tile_ceil[t] as f64 - 0.01 { return true; }
        }
        false
    }
    fn zone_of_tile(&self, t: usize) -> &crate::archgen::Zone {
        let l = self.layout.as_ref().expect("layout");
        &l.zones[l.zone_of[t] as usize]
    }
    /// line 3716 `wallMatKey(ownerTile)`
    pub fn wall_mat_key(&self, owner_tile: usize) -> String {
        let (x, z) = (owner_tile as i32 % self.w, owner_tile as i32 / self.w);
        let zn = self.zone_of_tile(owner_tile);
        let rid = if self.tiles[owner_tile] == 1 { self.wall_region_idx(x, z) } else { self.region_map[owner_tile] as usize };
        let info = &self.region_info[rid];
        if zn.wall_mat == "wallpaper" || info.flags & RF_MOSS != 0 { return format!("wp:{}", info.wall); }
        format!("{}:{}", zn.wall_mat, grime_of(info))
    }
    /// line 3724 `floorMatKey(t, rid)`
    pub fn floor_mat_key(&self, t: usize, rid: usize) -> String {
        let (zn, info) = (self.zone_of_tile(t), &self.region_info[rid]);
        if zn.floor_mat == "carpet" || info.flags & RF_MOSS != 0 { return format!("carpet:{}", info.floor); }
        format!("floor:{}:{}", zn.floor_mat, grime_of(info))
    }
    /// line 3729 `ceilMatKey(t)`
    pub fn ceil_mat_key(&self, t: usize) -> String {
        let zn = self.zone_of_tile(t);
        let info = &self.region_info[self.region_map[t] as usize];
        if zn.ceil_mat == "tiles" || info.flags & RF_MOSS != 0 { return format!("tiles:{}", self.ceil_region[t]); }
        format!("ceil:{}:{}", zn.ceil_mat, grime_of(info))
    }

    /// line 3746 `addInstanced(geo, mat, list, write)` — one InstancedMesh per
    /// 10×10-tile chunk (insertion order), flagged `arch`, added to the world
    pub fn add_instanced<T>(&mut self, geo: &Geo, mat: &Mat, list: &[T], write: ArchWrite<T>) -> Option<Obj> {
        if list.is_empty() { return None; }
        let mut chunks: Vec<Vec<Mat4>> = vec![];
        let mut by_chunk: HashMap<(i64, i64), usize> = HashMap::new();
        for it in list {
            let m = write(it);
            let mut cp = Vec3::ZERO;
            cp.set_from_matrix_position(&m);
            let key = ((self.to_u(cp.x) / ARCH_CHUNK).floor() as i64, (self.to_v(cp.z) / ARCH_CHUNK).floor() as i64);
            let i = *by_chunk.entry(key).or_insert_with(|| { chunks.push(vec![]); chunks.len() - 1 });
            chunks[i].push(m);
        }
        let mut last = None;
        for mats in chunks {
            let im = instanced_mesh(geo, mat, mats.len());
            for (i, m) in mats.iter().enumerate() { im.set_matrix_at(i, m); }
            // (im.computeBoundingSphere(): the renderer bounds instanced meshes itself)
            im.user().arch = true;
            self.world.add(&im);
            last = Some(im);
        }
        last
    }

    /// line 3746 `buildFloors(poolReserved)`
    pub fn build_floors(&mut self, pool_reserved: &HashSet<usize>) {
        self.ensure_surface_mats();
        let mut groups: Vec<(String, Vec<(i32, i32)>)> = vec![];
        let mut gi: HashMap<String, usize> = HashMap::new();
        let solids = self.solids.take();
        for z in 0..self.h {
            for x in 0..self.w {
                let t = self.t_idx(x, z);
                if pool_reserved.contains(&t) { continue; }
                if self.tiles[t] == 1 && (x == 0 || z == 0 || x == self.w - 1 || z == self.h - 1 || solids.as_ref().map(|s| self.tile_fully_solid(s, t)).unwrap_or(false)) { continue; }
                let rid = if self.tiles[t] == 1 { self.wall_region_idx(x, z) } else { self.region_map[t] as usize };
                let key = self.floor_mat_key(t, rid);
                let i = *gi.entry(key.clone()).or_insert_with(|| { groups.push((key, vec![])); groups.len() - 1 });
                groups[i].1.push((x, z));
            }
        }
        self.solids = solids;
        let geo = self.floor_tile_geo.clone();
        for (key, list) in groups {
            let mat = if let Some(rest) = key.strip_prefix("carpet:") { self.floor_mats[rest.parse::<usize>().unwrap_or(0)].clone() } else { self.arch_mat(&key) };
            let (w, h) = (self.w as f64, self.h as f64);
            self.add_instanced(&geo, &mat, &list, &|&(x, z): &(i32, i32)| { let mut m = Mat4::IDENTITY; m.make_translation((x as f64 - w / 2.0 + 0.5) * CELL, 0, (z as f64 - h / 2.0 + 0.5) * CELL); m });
        }
    }

    /// line 3763 `buildCeilings()` — ceiling quads at each tile's height, then the bulkheads
    pub fn build_ceilings(&mut self) {
        self.ensure_surface_mats();
        let mut groups: Vec<(String, Vec<usize>)> = vec![];
        let mut gi: HashMap<String, usize> = HashMap::new();
        let solids = self.solids.take();
        for z in 0..self.h {
            for x in 0..self.w {
                let t = self.t_idx(x, z);
                if self.tiles[t] == 1 && solids.as_ref().map(|s| self.tile_fully_solid(s, t)).unwrap_or(false) { continue; }
                let key = self.ceil_mat_key(t);
                let i = *gi.entry(key.clone()).or_insert_with(|| { groups.push((key, vec![])); groups.len() - 1 });
                groups[i].1.push(t);
            }
        }
        self.solids = solids;
        let geo = self.ceil_tile_geo.clone();
        let (wi, w, h) = (self.w, self.w as f64, self.h as f64);
        let tc = self.tile_ceil.clone();
        for (key, list) in groups {
            let mat = if let Some(rest) = key.strip_prefix("tiles:") { self.ceil_mats[rest.parse::<usize>().unwrap_or(0)].clone() } else { self.arch_mat(&key) };
            self.add_instanced(&geo, &mat, &list, &|&t: &usize| {
                let (x, z) = ((t as i32 % wi) as f64, (t as i32 / wi) as f64);
                let mut m = Mat4::IDENTITY; m.make_translation((x - w / 2.0 + 0.5) * CELL, tc[t] as f64, (z - h / 2.0 + 0.5) * CELL); m
            });
        }
        // bulkheads: the vertical drop wherever two neighbouring ceilings differ
        struct Bulk { x: f64, z: f64, y0: f64, y1: f64, rot: f64 }
        let mut bulk: Vec<(String, Vec<Bulk>)> = vec![];
        let mut bi: HashMap<String, usize> = HashMap::new();
        for z in 0..self.h {
            for x in 0..self.w {
                let a = self.t_idx(x, z);
                for (dx, dz) in [(1, 0), (0, 1)] {
                    let (nx, nz) = (x + dx, z + dz);
                    if !self.in_grid(nx, nz) { continue; }
                    let b = self.t_idx(nx, nz);
                    let (ha, hb) = (self.tile_ceil[a] as f64, self.tile_ceil[b] as f64);
                    if (ha - hb).abs() < 0.02 { continue; }
                    if self.tiles[a] == 1 && self.tiles[b] == 1 { continue; }   // buried between two walls
                    let (low, high) = if ha < hb { (a, b) } else { (b, a) };
                    let zn = self.zone_of_tile(low);
                    let g = grime_of(&self.region_info[self.region_map[low] as usize]);
                    let key = format!("{}:{}", if zn.ceil_mat == "slab" { "concrete" } else { "paint" }, g);
                    // normal points into the higher tile (the side it is seen from)
                    let sgn = if high == b { 1.0 } else { -1.0 };
                    let e = Bulk { x: (self.wx(x) + self.wx(nx)) / 2.0, z: (self.wz(z) + self.wz(nz)) / 2.0, y0: ha.min(hb), y1: ha.max(hb),
                        rot: if dx != 0 { sgn * PI / 2.0 } else if sgn > 0.0 { 0.0 } else { PI } };
                    let i = *bi.entry(key.clone()).or_insert_with(|| { bulk.push((key, vec![])); bulk.len() - 1 });
                    bulk[i].1.push(e);
                }
            }
        }
        let geo = self.unit_plane_geo.clone();
        for (key, list) in bulk {
            let mat = self.arch_mat(&key);
            self.add_instanced(&geo, &mat, &list, &|e: &Bulk| {
                let q = Quat::from_axis_angle(&Vec3::new(0.0, 1.0, 0.0), e.rot);
                Mat4::compose(&Vec3::new(e.x, (e.y0 + e.y1) / 2.0, e.z), &q, &Vec3::new(CELL, e.y1 - e.y0, 1.0))
            });
        }
    }

    /// line 3808 `buildWalls()` — one instanced unit box per archgen solid
    pub fn build_walls(&mut self) {
        let Some(s) = self.solids.take() else { return };
        let mut groups: Vec<(String, Vec<usize>)> = vec![];
        let mut gi: HashMap<String, usize> = HashMap::new();
        for b in 0..s.count {
            let key = self.wall_mat_key(s.boxes[b * 8 + 6] as usize);
            let i = *gi.entry(key.clone()).or_insert_with(|| { groups.push((key, vec![])); groups.len() - 1 });
            groups[i].1.push(b);
        }
        let geo = self.unit_box_geo.clone();
        let (w, h) = (self.w as f64, self.h as f64);
        let bx = s.boxes.clone();
        self.solids = Some(s);
        for (key, list) in groups {
            let mat = self.arch_mat(&key);
            self.add_instanced(&geo, &mat, &list, &|&b: &usize| {
                let o = b * 8;
                let f = |k: usize| bx[o + k] as f64;
                let (x0, x1) = ((f(0) - w / 2.0) * CELL, (f(2) - w / 2.0) * CELL);
                let (z0, z1) = ((f(1) - h / 2.0) * CELL, (f(3) - h / 2.0) * CELL);
                Mat4::compose(&Vec3::new((x0 + x1) / 2.0, (f(4) + f(5)) / 2.0, (z0 + z1) / 2.0), &Quat::IDENTITY,
                    &Vec3::new(0.01f64.max(x1 - x0), 0.01f64.max(f(5) - f(4)), 0.01f64.max(z1 - z0)))
            });
        }
    }

    /// line 3833 `layPanels()` — ceiling fixtures follow the local ceiling;
    /// under a tall slab the tubes hang on rods
    pub fn lay_panels(&mut self) {
        let Some(pm) = self.panel_mesh.clone() else { return };
        let mut rods: Vec<(f64, f64, f64, f64)> = vec![];
        for i in 0..self.panels.len() {
            let (pwx, pwz) = (self.panels[i].wx, self.panels[i].wz);
            let t = self.t_idx(self.tx(pwx), self.tz(pwz));
            let ceil = self.tile_ceil[t] as f64;
            let p = &mut self.panels[i];
            p.f.insert("ceil".into(), ceil);
            p.y = if ceil > FIXTURE_HANG + 0.3 { FIXTURE_HANG } else { ceil };
            if p.y < ceil - 0.05 { rods.push((p.wx, p.wz, p.y, ceil)); }
            p.h_boost = (1f64.max(p.y / WALL_H)).powf(1.35);
            let mut m = Mat4::IDENTITY; m.make_translation(p.wx, p.y - 0.04, p.wz);
            pm.set_matrix_at(i, &m);
        }
        // two hanger rods per hung fixture
        let mut list: Vec<(f64, f64, f64, f64)> = vec![];
        for &(x, z, y0, y1) in &rods { for s in [-0.8, 0.8] { list.push((x + s, z, y0, y1)); } }
        let geo = self.unit_box_geo.clone();
        let mat = self.mats.metal.clone();
        self.add_instanced(&geo, &mat, &list, &|&(x, z, y0, y1): &(f64, f64, f64, f64)| {
            Mat4::compose(&Vec3::new(x, (y0 + y1) / 2.0, z), &Quat::IDENTITY, &Vec3::new(0.025, y1 - y0, 0.025))
        });
    }

    /// line 3853 `freezeStatic()` — scenery that never moves stops updating its matrix
    pub fn freeze_static(&mut self) {
        let mut n = 0usize;
        if self.hash_has("nofreeze") { return; }
        for o in self.world.children() {
            { let u = &o.r().user; if !(u.static_prop || u.arch) { continue; } }
            o.traverse(&mut |c: &Obj| { c.update_matrix(); c.set_matrix_auto_update(false); n += 1; });
        }
        self.build_profile.insert("frozen".into(), n as f64);
    }
}

// ------------------------------------------------------------ level teardown (3870–3902)
/// `collectResources(root)` — the geometries, materials and textures a tree uses
/// (holding them keeps them alive: Rust frees by refcount once these drop)
#[derive(Default)]
pub struct Resources {
    pub geo: Vec<Geo>, pub mat: Vec<Mat>, pub tex: Vec<Tex>,
    geo_ids: HashSet<u64>, mat_ids: HashSet<u64>, tex_ids: HashSet<u64>,
}
impl Resources {
    pub fn add_geo(&mut self, g: &Geo) { if self.geo_ids.insert(g.id()) { self.geo.push(g.clone()); } }
    pub fn add_tex(&mut self, t: &Tex) { if self.tex_ids.insert(t.id()) { self.tex.push(t.clone()); } }
    pub fn add_mat(&mut self, m: &Mat) {
        if self.mat_ids.insert(m.id()) { self.mat.push(m.clone()); }
        // every texture-valued property of the material
        let ts: Vec<Tex> = { let r = m.r(); [&r.map, &r.alpha_map, &r.bump_map, &r.emissive_map].into_iter().flatten().cloned().collect() };
        for t in ts { self.add_tex(&t); }
    }
    pub fn has_geo(&self, g: &Geo) -> bool { self.geo_ids.contains(&g.id()) }
    pub fn has_mat(&self, m: &Mat) -> bool { self.mat_ids.contains(&m.id()) }
    pub fn has_tex(&self, t: &Tex) -> bool { self.tex_ids.contains(&t.id()) }
}
impl std::fmt::Debug for Resources {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "Resources({} geo, {} mat, {} tex)", self.geo.len(), self.mat.len(), self.tex.len()) }
}
/// line 3872 `collectResources(root, out)`
pub fn collect_resources(root: &Obj, out: &mut Resources) {
    root.traverse(&mut |o: &Obj| {
        if let Some(g) = o.try_geo() { out.add_geo(&g); }
        for m in o.mats() { out.add_mat(&m); }
        // Points / Sprite / Line materials (three keeps them in `material` too)
        let extra: Option<Mat> = { let n = o.r(); match &n.kind { Kind::Points { mat, .. } | Kind::Sprite { mat, .. } | Kind::Line { mat, .. } => Some(mat.clone()), _ => None } };
        if let Some(m) = extra { out.add_mat(&m); }
    });
}

impl Game {
    /// `disposeUnused(pendingDispose)` — free what the last level used. The
    /// last level's resources are held in `pending_dispose` (JS
    /// `pendingDispose`) until the new level is up; anything the live scene no
    /// longer references is counted in `dispose_stats` and released here (the
    /// renderer's GPU caches drop their copies on their own sweep, by weak ref).
    pub fn dispose_unused(&mut self) {
        let Some(old) = self.pending_dispose.take() else { return };
        if self.hash_has("nodispose") { return; }   // debug: #nodispose (JS keeps them alive by not disposing)
        let mut live = Resources::default();
        collect_resources(&self.scene, &mut live);
        // module-level shared resources that may be temporarily unreferenced
        for g in [&self.floor_tile_geo, &self.ceil_tile_geo, &self.unit_box_geo, &self.unit_plane_geo, &self.panel_geo, &self.wall_geo] { live.add_geo(g); }
        for m in self.arch_mats.values() { live.add_mat(m); }
        for m in self.floor_mats.iter().chain(self.ceil_mats.iter()).chain(self.wall_mats.iter()) { live.add_mat(m); }
        for g in &old.geo { if !live.has_geo(g) { g.dispose(); self.dispose_stats[0] += 1; } }
        for m in &old.mat { if !live.has_mat(m) { self.dispose_stats[1] += 1; } }
        for t in &old.tex { if !live.has_tex(t) { t.dispose(); self.dispose_stats[2] += 1; } }
        drop(old);
    }
    /// `warmUpGPU()` (asks the renderer to compile every program the level needs
    /// and upload its textures; the app does it before the next frame)
    pub fn warm_up_gpu(&mut self) { self.want_warm_up = true; }

    // ------------------------------------------------------------ prop draw distance (3903–3948)
    /// line 3912 `buildPropCull()`
    pub fn build_prop_cull(&mut self) {
        self.prop_cull.list.clear(); self.prop_cull.t = 0.0;
        self.prop_cull_xz.clear();
        if self.hash_has("nocull") { return; }
        for o in self.world.children() {
            { let u = &o.r().user; if !(u.placer.as_deref().map(|s| !s.is_empty()).unwrap_or(false) || u.instanced) { continue; } }
            if o.is_light() { continue; }
            let mut light = false;
            o.traverse(&mut |c: &Obj| { if c.is_light() { light = true; } });
            if light { continue; }
            let sph = if o.is_instanced() { instanced_bounding_sphere(&o) } else {
                let mut b = Box3::default(); b.set_from_object(&o);
                if b.is_empty() { continue; }
                b.get_bounding_sphere()
            };
            self.prop_cull.list.push((o.clone(), sph.radius));
            self.prop_cull_xz.push((sph.center.x, sph.center.z));
        }
    }
    /// `updatePropCull(dt)` — hide props past their draw distance, a few times a second
    pub fn update_prop_cull(&mut self, dt: f64) {
        if self.prop_cull.list.is_empty() { return; }
        self.prop_cull.t -= dt;
        if self.prop_cull.t > 0.0 { return; }
        self.prop_cull.t = 0.2;
        // screen-size culling: radius r spans ~1100·r/d px at 720p; below ~6 px it goes
        let cap = if self.level_scp.as_deref() == Some("store") { PROP_FAR_STORE } else { self.camera.camera().far };
        let cp = self.camera.pos();
        let mut hidden = 0;
        for (i, (o, r)) in self.prop_cull.list.iter().enumerate() {
            let (x, z) = self.prop_cull_xz[i];
            let d = crate::stairwell::js_hypot2(x - cp.x, z - cp.z) - r;
            let vis = d < cap.min((PROP_FAR * 0.4).max(190.0 * r));
            if o.visible() != vis { o.set_visible(vis); }
            if !vis { hidden += 1; }
        }
        self.prop_cull.hidden = hidden;
    }
}

/// three's `InstancedMesh.computeBoundingSphere()` (local space): the union
/// of the geometry's sphere under every instance matrix
pub fn instanced_bounding_sphere(o: &Obj) -> Sphere {
    let gs = o.geo().bounding_sphere();
    let mut out = Sphere { center: Vec3::ZERO, radius: -1.0 };
    for i in 0..o.instance_count() {
        let mut s = gs;
        s.apply_matrix4(&o.get_matrix_at(i));
        sphere_union(&mut out, &s);
    }
    out
}
/// three r160 `Sphere.union(sphere)`
pub fn sphere_union(a: &mut Sphere, s: &Sphere) {
    if s.radius < 0.0 { return; }
    if a.radius < 0.0 { *a = *s; return; }
    if a.center == s.center { a.radius = a.radius.max(s.radius); return; }
    // setLength(r) = normalize() (× 1/(len || 1)) then × r
    let v = s.center - a.center;
    let l = v.length();
    let v2 = v * (1.0 / if l == 0.0 { 1.0 } else { l }) * s.radius;
    sphere_expand_by_point(a, &(s.center + v2));
    sphere_expand_by_point(a, &(s.center - v2));
}
/// three r160 `Sphere.expandByPoint(point)`
pub fn sphere_expand_by_point(a: &mut Sphere, p: &Vec3) {
    if a.radius < 0.0 { a.center = *p; a.radius = 0.0; return; }
    let v1 = *p - a.center;
    let l2 = v1.length_sq();
    if l2 > a.radius * a.radius {
        let l = l2.sqrt();
        let delta = (l - a.radius) * 0.5;
        a.center = a.center + v1 * (delta / l);
        a.radius += delta;
    }
}
