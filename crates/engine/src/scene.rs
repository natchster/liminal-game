//! The scene graph: nodes with a transform (position, rotation/quaternion, scale)
//! and a kind: groups, meshes, lights, sprites, points, bones, cameras.

use crate::geometry::*;
use crate::material::*;
use crate::math::*;
use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::rc::{Rc, Weak};

#[derive(Clone, Debug, Default)]
pub struct LightData {
    pub color: Color,
    pub intensity: f64,
    pub distance: f64,
    pub decay: f64,
    /// SpotLight cone half-angle and penumbra
    pub angle: f64,
    pub penumbra: f64,
    /// HemisphereLight
    pub ground_color: Color,
    /// SpotLight / DirectionalLight target
    pub target: Option<Obj>,
}

#[derive(Clone, Debug, Default)]
pub struct CameraData {
    pub fov: f64,
    pub aspect: f64,
    pub near: f64,
    pub far: f64,
    pub zoom: f64,
}
impl CameraData {
    /// Perspective projection matrix from fov / aspect / near / far / zoom
    /// (OpenGL clip space, z in [-1, 1]).
    pub fn projection(&self) -> Mat4 {
        let top = self.near * (deg_to_rad(0.5 * self.fov)).tan() / self.zoom;
        let height = 2.0 * top;
        let width = self.aspect * height;
        let left = -0.5 * width;
        make_perspective(left, left + width, top, top - height, self.near, self.far)
    }
}
pub fn make_perspective(left: f64, right: f64, top: f64, bottom: f64, near: f64, far: f64) -> Mat4 {
    let x = 2.0 * near / (right - left);
    let y = 2.0 * near / (top - bottom);
    let a = (right + left) / (right - left);
    let b = (top + bottom) / (top - bottom);
    let c = -(far + near) / (far - near);
    let d = -2.0 * far * near / (far - near);
    let mut m = Mat4::IDENTITY;
    m.set(x, 0.0, a, 0.0, 0.0, y, b, 0.0, 0.0, 0.0, c, d, 0.0, 0.0, -1.0, 0.0);
    m
}

#[derive(Clone, Debug)]
pub struct InstanceData {
    pub count: usize,
    pub matrices: Vec<Mat4>,
    pub colors: Option<Vec<Color>>,
    /// Each instance's own wear by use (`set_use_wear_at`), when the
    /// instances were different things; None: all as the mesh's own
    /// (`Node::use_wear`)
    pub use_wear: Option<Vec<UseWear>>,
    pub version: u64,
    /// Bounding sphere over all instances (None = not computed yet)
    pub bounding_sphere: Option<Sphere>,
    /// Bounding box over all instances (None = not computed yet)
    pub bounding_box: Option<Box3>,
}

/// How a thing is worn by use, the thing's own (not its material's: a
/// material is shared by many things, `docs/design/PROPS.md` P4). It scales the
/// wear by use of a material that takes it (`Material::wear_use`): the game
/// sets it on every mesh of a prop as it settles the prop's look; anything
/// else keeps the default (handled as furniture, the floor at 0)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UseWear {
    /// how much it is handled: its edges rubbed, its low parts knocked and
    /// scuffed (1 furniture; 0.45 a thing hung out of reach, only dusted)
    pub handled: f64,
    /// how much its paint is chipped through (1 furniture; little on a
    /// small thing on the floor, scuffed rather than chipped; 0 hung up)
    pub chips: f64,
    /// the height of the floor under it, world: the low bands (the floor's
    /// grime at its foot, the mop's tide line, the scuffs) are measured
    /// from it, so a thing on an upper storey has them too
    pub floor: f64,
}
impl Default for UseWear {
    fn default() -> Self {
        UseWear { handled: 1.0, chips: 1.0, floor: 0.0 }
    }
}

#[derive(Clone, Debug)]
pub struct Skeleton {
    pub bones: Vec<Obj>,
    pub bone_inverses: Vec<Mat4>,
}

#[derive(Clone, Debug)]
pub enum Kind {
    Node,
    Group,
    Scene,
    Mesh { geo: Geo, mats: Vec<Mat> },
    Instanced { geo: Geo, mats: Vec<Mat>, inst: InstanceData },
    Skinned { geo: Geo, mats: Vec<Mat>, skeleton: Option<Skeleton>, bind_matrix: Mat4, bind_matrix_inverse: Mat4 },
    Points { geo: Geo, mat: Mat },
    Line { geo: Geo, mat: Mat, segments: bool, looped: bool },
    Sprite { mat: Mat, center: Vec2 },
    PointLight(LightData),
    SpotLight(LightData),
    AmbientLight(LightData),
    HemisphereLight(LightData),
    DirectionalLight(LightData),
    Bone,
    Camera(CameraData),
}

/// Free-form per-object data. Common keys are fields; the
/// rest go in the maps.
#[derive(Clone, Debug, Default)]
pub struct UserData {
    pub placer: Option<String>,
    pub static_prop: bool,
    pub floats: bool,
    pub gameplay: bool,
    pub no_instance: bool,
    pub instanced: bool,
    pub hide: bool,
    pub arch: bool,
    pub prop_part: bool,
    pub panel: Option<usize>,
    pub w: f64,
    pub phase: f64,
    pub base_x: f64,
    pub base_y: f64,
    pub base_z: f64,
    pub f: HashMap<String, f64>,
    pub s: HashMap<String, String>,
    pub o: HashMap<String, Obj>,
}
impl UserData {
    pub fn get(&self, k: &str) -> f64 {
        self.f.get(k).copied().unwrap_or(0.0)
    }
    pub fn has(&self, k: &str) -> bool {
        self.f.contains_key(k)
    }
    pub fn set(&mut self, k: &str, v: impl Num) {
        self.f.insert(k.to_string(), v.f());
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RotSrc {
    Euler,
    Quat,
}

#[derive(Debug)]
pub struct Node {
    pub id: u64,
    pub name: String,
    pub kind: Kind,
    position: Vec3,
    rotation: Euler,
    quaternion: Quat,
    rot_src: RotSrc,
    scale: Vec3,
    pub up: Vec3,
    pub matrix: Mat4,
    pub matrix_world: Mat4,
    pub matrix_auto_update: bool,
    pub matrix_world_needs_update: bool,
    pub visible: bool,
    pub frustum_culled: bool,
    pub render_order: i32,
    pub cast_shadow: bool,
    pub receive_shadow: bool,
    pub layers: u32,
    pub morph_influences: Vec<f64>,
    /// how it is worn by use (a mesh's; see [`UseWear`])
    pub use_wear: UseWear,
    pub children: Vec<Obj>,
    parent: Weak<RefCell<Node>>,
    pub user: UserData,
}

impl Node {
    fn new(kind: Kind) -> Node {
        Node {
            id: next_id(),
            name: String::new(),
            kind,
            position: Vec3::ZERO,
            rotation: Euler::default(),
            quaternion: Quat::IDENTITY,
            rot_src: RotSrc::Euler,
            scale: Vec3::new(1.0, 1.0, 1.0),
            up: Vec3::new(0.0, 1.0, 0.0),
            matrix: Mat4::IDENTITY,
            matrix_world: Mat4::IDENTITY,
            matrix_auto_update: true,
            matrix_world_needs_update: false,
            visible: true,
            frustum_culled: true,
            render_order: 0,
            cast_shadow: false,
            receive_shadow: false,
            layers: 1,
            morph_influences: vec![],
            use_wear: UseWear::default(),
            children: vec![],
            parent: Weak::new(),
            user: UserData::default(),
        }
    }
    fn sync_quat(&mut self) {
        if self.rot_src == RotSrc::Euler {
            self.quaternion.set_from_euler(&self.rotation);
        }
    }
    fn sync_euler(&mut self) {
        if self.rot_src == RotSrc::Quat {
            let q = self.quaternion;
            let o = self.rotation.order;
            self.rotation.set_from_quaternion(&q, o);
        }
    }
    pub fn quat(&mut self) -> Quat {
        self.sync_quat();
        self.quaternion
    }
    pub fn update_matrix(&mut self) {
        self.sync_quat();
        self.matrix = Mat4::compose(&self.position, &self.quaternion, &self.scale);
        self.matrix_world_needs_update = true;
    }
    pub fn geo(&self) -> Option<Geo> {
        match &self.kind {
            Kind::Mesh { geo, .. }
            | Kind::Instanced { geo, .. }
            | Kind::Skinned { geo, .. }
            | Kind::Points { geo, .. }
            | Kind::Line { geo, .. } => Some(geo.clone()),
            _ => None,
        }
    }
    pub fn mats(&self) -> Vec<Mat> {
        match &self.kind {
            Kind::Mesh { mats, .. } | Kind::Instanced { mats, .. } | Kind::Skinned { mats, .. } => mats.clone(),
            Kind::Points { mat, .. } | Kind::Line { mat, .. } | Kind::Sprite { mat, .. } => vec![mat.clone()],
            _ => vec![],
        }
    }
    pub fn light(&self) -> Option<&LightData> {
        match &self.kind {
            Kind::PointLight(l) | Kind::SpotLight(l) | Kind::AmbientLight(l) | Kind::HemisphereLight(l) | Kind::DirectionalLight(l) => {
                Some(l)
            }
            _ => None,
        }
    }
    pub fn light_mut(&mut self) -> Option<&mut LightData> {
        match &mut self.kind {
            Kind::PointLight(l) | Kind::SpotLight(l) | Kind::AmbientLight(l) | Kind::HemisphereLight(l) | Kind::DirectionalLight(l) => {
                Some(l)
            }
            _ => None,
        }
    }
}

/// Shared scene-node handle. Cloning the handle copies the reference (both
/// handles see the same node); `duplicate()` makes a new node.
#[derive(Clone, Debug)]
pub struct Obj(pub Rc<RefCell<Node>>);
impl PartialEq for Obj {
    fn eq(&self, o: &Obj) -> bool {
        Rc::ptr_eq(&self.0, &o.0)
    }
}
impl Eq for Obj {}
impl std::hash::Hash for Obj {
    fn hash<H: std::hash::Hasher>(&self, h: &mut H) {
        (Rc::as_ptr(&self.0) as usize).hash(h)
    }
}

/// Guard over a node's rotation that keeps the quaternion in sync when dropped.
pub struct RotGuard<'a>(RefMut<'a, Node>);
impl Deref for RotGuard<'_> {
    type Target = Euler;
    fn deref(&self) -> &Euler {
        &self.0.rotation
    }
}
impl DerefMut for RotGuard<'_> {
    fn deref_mut(&mut self) -> &mut Euler {
        &mut self.0.rotation
    }
}
/// Guard over a node's quaternion that keeps the Euler rotation in sync.
pub struct QuatGuard<'a>(RefMut<'a, Node>);
impl Deref for QuatGuard<'_> {
    type Target = Quat;
    fn deref(&self) -> &Quat {
        &self.0.quaternion
    }
}
impl DerefMut for QuatGuard<'_> {
    fn deref_mut(&mut self) -> &mut Quat {
        &mut self.0.quaternion
    }
}

impl Obj {
    pub fn with_kind(kind: Kind) -> Obj {
        Obj(Rc::new(RefCell::new(Node::new(kind))))
    }
    pub fn n(&self) -> RefMut<'_, Node> {
        self.0.borrow_mut()
    }
    pub fn r(&self) -> Ref<'_, Node> {
        self.0.borrow()
    }
    pub fn id(&self) -> u64 {
        self.0.borrow().id
    }
    pub fn ptr_eq(&self, o: &Obj) -> bool {
        Rc::ptr_eq(&self.0, &o.0)
    }

    // ------------------------------------------------ transform (mutable guards)
    /// The position for mutation: `o.position().set(1, 2, 3)`, `o.position().y += 0.1`.
    pub fn position(&self) -> RefMut<'_, Vec3> {
        RefMut::map(self.0.borrow_mut(), |n| &mut n.position)
    }
    /// The scale for mutation.
    pub fn scale(&self) -> RefMut<'_, Vec3> {
        RefMut::map(self.0.borrow_mut(), |n| &mut n.scale)
    }
    /// The Euler rotation for mutation (the quaternion follows).
    pub fn rotation(&self) -> RotGuard<'_> {
        let mut n = self.0.borrow_mut();
        n.sync_euler();
        n.rot_src = RotSrc::Euler;
        RotGuard(n)
    }
    /// The quaternion for mutation (the Euler rotation follows).
    pub fn quaternion(&self) -> QuatGuard<'_> {
        let mut n = self.0.borrow_mut();
        n.sync_quat();
        n.rot_src = RotSrc::Quat;
        QuatGuard(n)
    }
    /// The node's user data for mutation.
    pub fn user(&self) -> RefMut<'_, UserData> {
        RefMut::map(self.0.borrow_mut(), |n| &mut n.user)
    }
    // ------------------------------------------------ transform (copies, safe anywhere)
    pub fn pos(&self) -> Vec3 {
        self.0.borrow().position
    }
    pub fn scl(&self) -> Vec3 {
        self.0.borrow().scale
    }
    pub fn rot(&self) -> Euler {
        let mut n = self.0.borrow_mut();
        n.sync_euler();
        n.rotation
    }
    pub fn quat(&self) -> Quat {
        self.0.borrow_mut().quat()
    }
    pub fn set_pos(&self, x: impl Num, y: impl Num, z: impl Num) -> &Self {
        self.0.borrow_mut().position = v3(x, y, z);
        self
    }
    pub fn set_pos_v(&self, p: &Vec3) -> &Self {
        self.0.borrow_mut().position = *p;
        self
    }
    pub fn set_scale(&self, x: impl Num, y: impl Num, z: impl Num) -> &Self {
        self.0.borrow_mut().scale = v3(x, y, z);
        self
    }
    pub fn set_scalar(&self, s: impl Num) -> &Self {
        let s = s.f();
        self.0.borrow_mut().scale = Vec3::new(s, s, s);
        self
    }
    pub fn set_rot(&self, x: impl Num, y: impl Num, z: impl Num) -> &Self {
        let mut r = self.rotation();
        r.x = x.f();
        r.y = y.f();
        r.z = z.f();
        drop(r);
        self
    }
    pub fn set_quat(&self, q: &Quat) -> &Self {
        *self.quaternion() = *q;
        self
    }
    pub fn set_rotation_order(&self, o: EulerOrder) {
        let mut r = self.rotation();
        r.order = o;
    }
    pub fn up(&self) -> Vec3 {
        self.0.borrow().up
    }

    // ------------------------------------------------ flags
    pub fn visible(&self) -> bool {
        self.0.borrow().visible
    }
    pub fn set_visible(&self, v: bool) -> &Self {
        self.0.borrow_mut().visible = v;
        self
    }
    /// How it is worn by use (see [`UseWear`])
    pub fn use_wear(&self) -> UseWear {
        self.0.borrow().use_wear
    }
    pub fn set_use_wear(&self, w: UseWear) -> &Self {
        self.0.borrow_mut().use_wear = w;
        self
    }
    pub fn name(&self) -> String {
        self.0.borrow().name.clone()
    }
    pub fn set_name(&self, s: &str) -> &Self {
        self.0.borrow_mut().name = s.to_string();
        self
    }
    pub fn set_render_order(&self, v: i32) -> &Self {
        self.0.borrow_mut().render_order = v;
        self
    }
    pub fn set_frustum_culled(&self, v: bool) -> &Self {
        self.0.borrow_mut().frustum_culled = v;
        self
    }
    pub fn set_matrix_auto_update(&self, v: bool) -> &Self {
        self.0.borrow_mut().matrix_auto_update = v;
        self
    }

    // ------------------------------------------------ hierarchy
    pub fn add(&self, child: &Obj) -> &Self {
        if child.ptr_eq(self) {
            return self;
        }
        child.remove_from_parent();
        child.0.borrow_mut().parent = Rc::downgrade(&self.0);
        self.0.borrow_mut().children.push(child.clone());
        self
    }
    pub fn add_all(&self, kids: &[&Obj]) -> &Self {
        for k in kids {
            self.add(k);
        }
        self
    }
    pub fn remove(&self, child: &Obj) -> &Self {
        let mut n = self.0.borrow_mut();
        if let Some(i) = n.children.iter().position(|c| c.ptr_eq(child)) {
            let c = n.children.remove(i);
            drop(n);
            c.0.borrow_mut().parent = Weak::new();
        }
        self
    }
    pub fn remove_from_parent(&self) -> &Self {
        if let Some(p) = self.parent() {
            p.remove(self);
        }
        self
    }
    pub fn clear(&self) -> &Self {
        let kids = std::mem::take(&mut self.0.borrow_mut().children);
        for k in kids {
            k.0.borrow_mut().parent = Weak::new();
        }
        self
    }
    pub fn parent(&self) -> Option<Obj> {
        self.0.borrow().parent.upgrade().map(Obj)
    }
    pub fn children(&self) -> Vec<Obj> {
        self.0.borrow().children.clone()
    }
    pub fn child(&self, i: usize) -> Option<Obj> {
        self.0.borrow().children.get(i).cloned()
    }
    pub fn child_count(&self) -> usize {
        self.0.borrow().children.len()
    }
    /// Depth-first, parent before children.
    pub fn traverse(&self, f: &mut dyn FnMut(&Obj)) {
        f(self);
        let kids = self.children();
        for k in &kids {
            k.traverse(f);
        }
    }
    pub fn traverse_visible(&self, f: &mut dyn FnMut(&Obj)) {
        if !self.visible() {
            return;
        }
        f(self);
        let kids = self.children();
        for k in &kids {
            k.traverse_visible(f);
        }
    }
    pub fn get_object_by_name(&self, name: &str) -> Option<Obj> {
        if self.0.borrow().name == name {
            return Some(self.clone());
        }
        for k in self.children() {
            if let Some(o) = k.get_object_by_name(name) {
                return Some(o);
            }
        }
        None
    }

    // ------------------------------------------------ kind accessors
    pub fn is_mesh(&self) -> bool {
        matches!(self.0.borrow().kind, Kind::Mesh { .. } | Kind::Instanced { .. } | Kind::Skinned { .. })
    }
    pub fn is_instanced(&self) -> bool {
        matches!(self.0.borrow().kind, Kind::Instanced { .. })
    }
    pub fn is_light(&self) -> bool {
        self.0.borrow().light().is_some()
    }
    pub fn is_point_light(&self) -> bool {
        matches!(self.0.borrow().kind, Kind::PointLight(_))
    }
    pub fn is_spot_light(&self) -> bool {
        matches!(self.0.borrow().kind, Kind::SpotLight(_))
    }
    pub fn is_sprite(&self) -> bool {
        matches!(self.0.borrow().kind, Kind::Sprite { .. })
    }
    pub fn is_points(&self) -> bool {
        matches!(self.0.borrow().kind, Kind::Points { .. })
    }
    pub fn is_group(&self) -> bool {
        matches!(self.0.borrow().kind, Kind::Group | Kind::Node | Kind::Scene)
    }
    pub fn is_bone(&self) -> bool {
        matches!(self.0.borrow().kind, Kind::Bone)
    }
    pub fn is_camera(&self) -> bool {
        matches!(self.0.borrow().kind, Kind::Camera(_))
    }
    /// The geometry (panics if the object has none).
    pub fn geo(&self) -> Geo {
        self.0.borrow().geo().expect("object has no geometry")
    }
    pub fn try_geo(&self) -> Option<Geo> {
        self.0.borrow().geo()
    }
    /// The material (the first one for multi-material meshes)
    pub fn mat(&self) -> Mat {
        self.0.borrow().mats().into_iter().next().expect("object has no material")
    }
    pub fn try_mat(&self) -> Option<Mat> {
        self.0.borrow().mats().into_iter().next()
    }
    pub fn mats(&self) -> Vec<Mat> {
        self.0.borrow().mats()
    }
    /// Replace the material(s).
    pub fn set_mat(&self, m: impl IntoMats) -> &Self {
        let ms = m.into_mats();
        let mut n = self.0.borrow_mut();
        match &mut n.kind {
            Kind::Mesh { mats, .. } | Kind::Instanced { mats, .. } | Kind::Skinned { mats, .. } => *mats = ms,
            Kind::Points { mat, .. } | Kind::Line { mat, .. } | Kind::Sprite { mat, .. } => *mat = ms[0].clone(),
            _ => {}
        }
        drop(n);
        self
    }
    /// Replace the geometry.
    pub fn set_geo(&self, g: impl Into<GeoArg>) -> &Self {
        let g = g.into().0;
        let mut n = self.0.borrow_mut();
        match &mut n.kind {
            Kind::Mesh { geo, .. }
            | Kind::Instanced { geo, .. }
            | Kind::Skinned { geo, .. }
            | Kind::Points { geo, .. }
            | Kind::Line { geo, .. } => *geo = g,
            _ => {}
        }
        drop(n);
        self
    }
    /// Light parameters for mutation: `l.light().intensity = 2.0`.
    pub fn light(&self) -> RefMut<'_, LightData> {
        RefMut::map(self.0.borrow_mut(), |n| n.light_mut().expect("not a light"))
    }
    pub fn intensity(&self) -> f64 {
        self.0.borrow().light().map_or(0.0, |l| l.intensity)
    }
    pub fn set_intensity(&self, v: impl Num) -> &Self {
        if let Some(l) = self.0.borrow_mut().light_mut() {
            l.intensity = v.f();
        }
        self
    }
    pub fn light_color(&self) -> Color {
        self.0.borrow().light().map_or_default(|l| l.color)
    }
    /// The spot / directional light's target node.
    pub fn target(&self) -> Option<Obj> {
        self.0.borrow().light().and_then(|l| l.target.clone())
    }
    pub fn camera(&self) -> RefMut<'_, CameraData> {
        RefMut::map(self.0.borrow_mut(), |n| match &mut n.kind {
            Kind::Camera(c) => c,
            _ => panic!("not a camera"),
        })
    }
    pub fn morph_influences(&self) -> RefMut<'_, Vec<f64>> {
        RefMut::map(self.0.borrow_mut(), |n| &mut n.morph_influences)
    }

    // ------------------------------------------------ instancing
    pub fn set_matrix_at(&self, i: usize, m: &Mat4) {
        if let Kind::Instanced { inst, .. } = &mut self.0.borrow_mut().kind
            && i < inst.matrices.len()
        {
            inst.matrices[i] = *m;
            inst.version += 1;
        }
    }
    pub fn get_matrix_at(&self, i: usize) -> Mat4 {
        if let Kind::Instanced { inst, .. } = &self.0.borrow().kind {
            return inst.matrices.get(i).copied().unwrap_or(Mat4::IDENTITY);
        }
        Mat4::IDENTITY
    }
    pub fn set_color_at(&self, i: usize, c: &Color) {
        if let Kind::Instanced { inst, .. } = &mut self.0.borrow_mut().kind {
            let n = inst.matrices.len();
            let cols = inst.colors.get_or_insert_with(|| vec![Color::white(); n]);
            if i < cols.len() {
                cols[i] = *c;
            }
            inst.version += 1;
        }
    }
    /// Instance `i`'s own wear by use (the rest as the mesh's own until set)
    pub fn set_use_wear_at(&self, i: usize, w: UseWear) {
        let mut n = self.0.borrow_mut();
        let own = n.use_wear;
        if let Kind::Instanced { inst, .. } = &mut n.kind {
            let len = inst.matrices.len();
            let ws = inst.use_wear.get_or_insert_with(|| vec![own; len]);
            if i < ws.len() {
                ws[i] = w;
            }
            inst.version += 1;
        }
    }
    /// Instance `i`'s wear by use
    pub fn use_wear_at(&self, i: usize) -> UseWear {
        let n = self.0.borrow();
        if let Kind::Instanced { inst, .. } = &n.kind
            && let Some(w) = inst.use_wear.as_ref().and_then(|ws| ws.get(i))
        {
            return *w;
        }
        n.use_wear
    }
    /// Bounding sphere of an instanced mesh: the union of the geometry's
    /// sphere under every instance matrix (cached until computed again)
    pub fn compute_instanced_bounding_sphere(&self) -> Sphere {
        let Some(geo) = self.0.borrow().geo() else { return Sphere::default() };
        let gs = geo.bounding_sphere();
        let mut n = self.0.borrow_mut();
        let Kind::Instanced { inst, .. } = &mut n.kind else { return Sphere::default() };
        let mut b = Sphere::default();
        for m in inst.matrices.iter().take(inst.count) {
            let mut s = gs;
            s.apply_matrix4(m);
            b.union(&s);
        }
        inst.bounding_sphere = Some(b);
        b
    }
    /// Bounding box of an instanced mesh: the union of the geometry's box under
    /// every instance matrix (cached until computed again)
    pub fn compute_instanced_bounding_box(&self) -> Box3 {
        let Some(geo) = self.0.borrow().geo() else { return Box3::empty() };
        let gb = geo.bounding_box();
        let mut n = self.0.borrow_mut();
        let Kind::Instanced { inst, .. } = &mut n.kind else { return Box3::empty() };
        let mut b = Box3::empty();
        for m in inst.matrices.iter().take(inst.count) {
            let mut x = gb;
            x.apply_matrix4(m);
            b.union(&x);
        }
        inst.bounding_box = Some(b);
        b
    }
    /// The instanced mesh's bounding box, computed on first use (None for other objects)
    pub fn instanced_bounding_box(&self) -> Option<Box3> {
        let cached = match &self.0.borrow().kind {
            Kind::Instanced { inst, .. } => Some(inst.bounding_box),
            _ => None,
        };
        match cached {
            None => None,
            Some(Some(b)) => Some(b),
            Some(None) => Some(self.compute_instanced_bounding_box()),
        }
    }
    pub fn set_count(&self, c: usize) {
        if let Kind::Instanced { inst, .. } = &mut self.0.borrow_mut().kind {
            inst.count = c.min(inst.matrices.len());
            inst.version += 1;
        }
    }
    pub fn instance_count(&self) -> usize {
        if let Kind::Instanced { inst, .. } = &self.0.borrow().kind { inst.count } else { 0 }
    }
    pub fn instances_changed(&self) {
        if let Kind::Instanced { inst, .. } = &mut self.0.borrow_mut().kind {
            inst.version += 1;
        }
    }

    // ------------------------------------------------ matrices
    pub fn update_matrix(&self) {
        self.0.borrow_mut().update_matrix();
    }
    /// Update the local and world matrices of this node and its subtree (only
    /// the dirty ones unless `force`).
    pub fn update_matrix_world(&self, force: bool) {
        let parent_world = self.parent().map(|p| p.0.borrow().matrix_world);
        update_world_rec(self, parent_world.as_ref(), force);
    }
    /// Recompute this node's world matrix, optionally the ancestors' first and
    /// the descendants' after.
    pub fn update_world_matrix(&self, update_parents: bool, update_children: bool) {
        let parent = self.parent();
        if update_parents && let Some(p) = &parent {
            p.update_world_matrix(true, false);
        }
        {
            let pw = parent.map(|p| p.0.borrow().matrix_world);
            let mut n = self.0.borrow_mut();
            if n.matrix_auto_update {
                n.update_matrix();
            }
            n.matrix_world = match pw {
                Some(pw) => Mat4::mul(&pw, &n.matrix),
                None => n.matrix,
            };
        }
        if update_children {
            for c in self.children() {
                c.update_world_matrix(false, true);
            }
        }
    }
    pub fn matrix_world(&self) -> Mat4 {
        self.0.borrow().matrix_world
    }
    pub fn matrix(&self) -> Mat4 {
        self.0.borrow().matrix
    }
    pub fn set_matrix(&self, m: &Mat4) {
        let mut n = self.0.borrow_mut();
        n.matrix = *m;
        n.matrix_world_needs_update = true;
    }
    /// World-space position (updates the world matrices up the chain first)
    pub fn world_position(&self) -> Vec3 {
        self.update_world_matrix(true, false);
        let m = self.matrix_world();
        Vec3::new(m.e[12], m.e[13], m.e[14])
    }
    pub fn world_quaternion(&self) -> Quat {
        self.update_world_matrix(true, false);
        self.matrix_world().decompose().1
    }
    pub fn world_scale(&self) -> Vec3 {
        self.update_world_matrix(true, false);
        self.matrix_world().decompose().2
    }
    /// World-space facing: +Z of the world matrix (−Z for cameras)
    pub fn world_direction(&self) -> Vec3 {
        self.update_world_matrix(true, false);
        let e = self.matrix_world().e;
        let mut v = Vec3::new(e[8], e[9], e[10]);
        v.normalize();
        if self.is_camera() { -v } else { v }
    }
    pub fn local_to_world(&self, v: &Vec3) -> Vec3 {
        self.update_world_matrix(true, false);
        let mut p = *v;
        p.apply_matrix4(&self.matrix_world());
        p
    }
    pub fn world_to_local(&self, v: &Vec3) -> Vec3 {
        self.update_world_matrix(true, false);
        let mut p = *v;
        p.apply_matrix4(&self.matrix_world().inverted());
        p
    }
    /// Rotate to face a world point (cameras and lights look down -Z, others +Z)
    pub fn look_at(&self, x: impl Num, y: impl Num, z: impl Num) {
        let target = v3(x, y, z);
        self.update_world_matrix(true, false);
        let mw = self.matrix_world();
        let position = Vec3::new(mw.e[12], mw.e[13], mw.e[14]);
        let (is_cam_or_light, up) = {
            let n = self.0.borrow();
            (matches!(n.kind, Kind::Camera(_)) || n.light().is_some(), n.up)
        };
        let mut m1 = Mat4::IDENTITY;
        if is_cam_or_light {
            m1.look_at(&position, &target, &up);
        } else {
            m1.look_at(&target, &position, &up);
        }
        let mut q = Quat::IDENTITY;
        q.set_from_rotation_matrix(&m1);
        if let Some(p) = self.parent() {
            let mut r = Mat4::IDENTITY;
            r.extract_rotation(&p.matrix_world());
            let mut q1 = Quat::IDENTITY;
            q1.set_from_rotation_matrix(&r);
            q1.invert();
            q.premultiply(&q1);
        }
        *self.quaternion() = q;
    }
    pub fn look_at_v(&self, t: &Vec3) {
        self.look_at(t.x, t.y, t.z)
    }
    pub fn rotate_on_axis(&self, axis: &Vec3, angle: impl Num) {
        let q1 = Quat::from_axis_angle(axis, angle.f());
        self.quaternion().multiply(&q1);
    }
    pub fn rotate_on_world_axis(&self, axis: &Vec3, angle: impl Num) {
        let q1 = Quat::from_axis_angle(axis, angle.f());
        self.quaternion().premultiply(&q1);
    }
    pub fn rotate_x(&self, a: impl Num) {
        self.rotate_on_axis(&Vec3::new(1.0, 0.0, 0.0), a)
    }
    pub fn rotate_y(&self, a: impl Num) {
        self.rotate_on_axis(&Vec3::new(0.0, 1.0, 0.0), a)
    }
    pub fn rotate_z(&self, a: impl Num) {
        self.rotate_on_axis(&Vec3::new(0.0, 0.0, 1.0), a)
    }
    pub fn translate_on_axis(&self, axis: &Vec3, d: impl Num) {
        let mut v = *axis;
        v.apply_quaternion(&self.quat());
        let d = d.f();
        self.position().add_scaled_vector(&v, d);
    }
    pub fn translate_x(&self, d: impl Num) {
        self.translate_on_axis(&Vec3::new(1.0, 0.0, 0.0), d)
    }
    pub fn translate_y(&self, d: impl Num) {
        self.translate_on_axis(&Vec3::new(0.0, 1.0, 0.0), d)
    }
    pub fn translate_z(&self, d: impl Num) {
        self.translate_on_axis(&Vec3::new(0.0, 0.0, 1.0), d)
    }
    pub fn apply_matrix4(&self, m: &Mat4) {
        let mut n = self.0.borrow_mut();
        if n.matrix_auto_update {
            n.update_matrix();
        }
        let nm = Mat4::mul(m, &n.matrix);
        let (p, q, s) = nm.decompose();
        n.position = p;
        n.quaternion = q;
        n.rot_src = RotSrc::Quat;
        n.scale = s;
        n.matrix = nm;
    }
    pub fn apply_quaternion(&self, q: &Quat) {
        self.quaternion().premultiply(q);
    }

    /// A new object sharing geometry and materials,
    /// with every child cloned too.
    pub fn duplicate(&self) -> Obj {
        let src = self.0.borrow();
        let mut kind = src.kind.clone();
        if let Kind::SpotLight(l) | Kind::DirectionalLight(l) = &mut kind {
            // spot / directional lights get their own copy of the target
            if let Some(t) = &l.target {
                l.target = Some(t.duplicate());
            }
        }
        let mut n = Node::new(kind);
        n.name = src.name.clone();
        // copy the rotation order, then the quaternion; the Euler angles are
        // re-derived from the quaternion
        let q = if src.rot_src == RotSrc::Euler { Quat::from_euler(&src.rotation) } else { src.quaternion };
        n.position = src.position;
        n.rotation = Euler::with_order(0.0, 0.0, 0.0, src.rotation.order);
        n.quaternion = q;
        n.rot_src = RotSrc::Quat;
        n.scale = src.scale;
        n.up = src.up;
        n.matrix = src.matrix;
        n.matrix_world = src.matrix_world;
        n.matrix_auto_update = src.matrix_auto_update;
        n.visible = src.visible;
        n.frustum_culled = src.frustum_culled;
        n.render_order = src.render_order;
        n.layers = src.layers;
        n.morph_influences = src.morph_influences.clone();
        n.use_wear = src.use_wear;
        n.user = src.user.clone();
        let kids = src.children.clone();
        drop(src);
        let o = Obj(Rc::new(RefCell::new(n)));
        for k in kids {
            o.add(&k.duplicate());
        }
        o
    }
}

fn update_world_rec(o: &Obj, parent_world: Option<&Mat4>, force: bool) {
    let mut force = force;
    let mw;
    {
        let mut n = o.0.borrow_mut();
        if n.matrix_auto_update {
            n.update_matrix();
        }
        if n.matrix_world_needs_update || force {
            n.matrix_world = match parent_world {
                Some(pw) => Mat4::mul(pw, &n.matrix),
                None => n.matrix,
            };
            n.matrix_world_needs_update = false;
            force = true;
        }
        mw = n.matrix_world;
    }
    let kids = o.0.borrow().children.clone();
    for k in &kids {
        update_world_rec(k, Some(&mw), force);
    }
}

/// Geometry argument (`Geo` or `&Geo`).
pub struct GeoArg(pub Geo);
impl From<Geo> for GeoArg {
    fn from(g: Geo) -> Self {
        GeoArg(g)
    }
}
impl From<&Geo> for GeoArg {
    fn from(g: &Geo) -> Self {
        GeoArg(g.clone())
    }
}

// ===================================================================== constructors
/// An empty group.
pub fn group() -> Obj {
    Obj::with_kind(Kind::Group)
}
/// A plain transform node.
pub fn node() -> Obj {
    Obj::with_kind(Kind::Node)
}
/// A scene root.
pub fn new_scene() -> Obj {
    Obj::with_kind(Kind::Scene)
}
/// A mesh from a geometry and one or more materials.
pub fn mesh(g: impl Into<GeoArg>, m: impl IntoMats) -> Obj {
    Obj::with_kind(Kind::Mesh { geo: g.into().0, mats: m.into_mats() })
}
/// An instanced mesh with `count` instances.
pub fn instanced_mesh(g: impl Into<GeoArg>, m: impl IntoMats, count: usize) -> Obj {
    Obj::with_kind(Kind::Instanced {
        geo: g.into().0,
        mats: m.into_mats(),
        inst: InstanceData {
            count,
            matrices: vec![Mat4::IDENTITY; count],
            colors: None,
            use_wear: None,
            version: 1,
            bounding_sphere: None,
            bounding_box: None,
        },
    })
}
/// A skinned mesh (bind a skeleton with `bind_skeleton`).
pub fn skinned_mesh(g: impl Into<GeoArg>, m: impl IntoMats) -> Obj {
    Obj::with_kind(Kind::Skinned {
        geo: g.into().0,
        mats: m.into_mats(),
        skeleton: None,
        bind_matrix: Mat4::IDENTITY,
        bind_matrix_inverse: Mat4::IDENTITY,
    })
}
/// Bind a skeleton of `bones` to a skinned mesh — computes the bone inverses from the bones' current world matrices.
pub fn bind_skeleton(o: &Obj, bones: Vec<Obj>) {
    o.update_matrix_world(true);
    for b in &bones {
        b.update_world_matrix(true, false);
    }
    let inverses: Vec<Mat4> = bones.iter().map(|b| b.matrix_world().inverted()).collect();
    let mw = o.matrix_world();
    if let Kind::Skinned { skeleton, bind_matrix, bind_matrix_inverse, .. } = &mut o.0.borrow_mut().kind {
        *skeleton = Some(Skeleton { bones, bone_inverses: inverses });
        *bind_matrix = mw;
        *bind_matrix_inverse = mw.inverted();
    }
}
/// A skeleton bone.
pub fn bone() -> Obj {
    Obj::with_kind(Kind::Bone)
}
/// A point cloud.
pub fn points(g: impl Into<GeoArg>, m: &Mat) -> Obj {
    Obj::with_kind(Kind::Points { geo: g.into().0, mat: m.clone() })
}
/// A line strip.
pub fn line(g: impl Into<GeoArg>, m: &Mat) -> Obj {
    Obj::with_kind(Kind::Line { geo: g.into().0, mat: m.clone(), segments: false, looped: false })
}
/// Disjoint line segments (vertex pairs).
pub fn line_segments(g: impl Into<GeoArg>, m: &Mat) -> Obj {
    Obj::with_kind(Kind::Line { geo: g.into().0, mat: m.clone(), segments: true, looped: false })
}
/// A camera-facing sprite.
pub fn sprite(m: &Mat) -> Obj {
    Obj::with_kind(Kind::Sprite { mat: m.clone(), center: Vec2::new(0.5, 0.5) })
}
/// Point light (defaults via `point_light!`: intensity 1, distance 0, decay 2).
pub fn point_light(c: impl IntoColor, intensity: impl Num, distance: impl Num, decay: impl Num) -> Obj {
    Obj::with_kind(Kind::PointLight(LightData {
        color: c.into_color(),
        intensity: intensity.f(),
        distance: distance.f(),
        decay: decay.f(),
        ..Default::default()
    }))
}
/// Spot light (defaults via `spot_light!`: intensity 1, distance 0, angle π/3,
/// penumbra 0, decay 2).
pub fn spot_light(c: impl IntoColor, intensity: impl Num, distance: impl Num, angle: impl Num, penumbra: impl Num, decay: impl Num) -> Obj {
    let t = node();
    Obj::with_kind(Kind::SpotLight(LightData {
        color: c.into_color(),
        intensity: intensity.f(),
        distance: distance.f(),
        decay: decay.f(),
        angle: angle.f(),
        penumbra: penumbra.f(),
        target: Some(t),
        ..Default::default()
    }))
}
/// Ambient light.
pub fn ambient_light(c: impl IntoColor, intensity: impl Num) -> Obj {
    Obj::with_kind(Kind::AmbientLight(LightData { color: c.into_color(), intensity: intensity.f(), ..Default::default() }))
}
/// Hemisphere light (sky colour above, ground colour below).
pub fn hemisphere_light(sky: impl IntoColor, ground: impl IntoColor, intensity: impl Num) -> Obj {
    let o = Obj::with_kind(Kind::HemisphereLight(LightData {
        color: sky.into_color(),
        ground_color: ground.into_color(),
        intensity: intensity.f(),
        ..Default::default()
    }));
    o.set_pos(0, 1, 0);
    o
}
/// Directional light.
pub fn directional_light(c: impl IntoColor, intensity: impl Num) -> Obj {
    let o = Obj::with_kind(Kind::DirectionalLight(LightData {
        color: c.into_color(),
        intensity: intensity.f(),
        target: Some(node()),
        ..Default::default()
    }));
    o.set_pos(0, 1, 0);
    o
}
/// Perspective camera (vertical fov in degrees).
pub fn perspective_camera(fov: f64, aspect: f64, near: f64, far: f64) -> Obj {
    Obj::with_kind(Kind::Camera(CameraData { fov, aspect, near, far, zoom: 1.0 }))
}

/// `point_light!(color[, intensity[, distance[, decay]]])` with defaults for
/// the omitted arguments.
#[macro_export]
macro_rules! point_light {
    ($c:expr_2021) => {
        $crate::scene::point_light($c, 1.0, 0.0, 2.0)
    };
    ($c:expr_2021, $i:expr_2021) => {
        $crate::scene::point_light($c, $i, 0.0, 2.0)
    };
    ($c:expr_2021, $i:expr_2021, $d:expr_2021) => {
        $crate::scene::point_light($c, $i, $d, 2.0)
    };
    ($c:expr_2021, $i:expr_2021, $d:expr_2021, $k:expr_2021) => {
        $crate::scene::point_light($c, $i, $d, $k)
    };
}
#[macro_export]
macro_rules! spot_light {
    ($c:expr_2021) => {
        $crate::scene::spot_light($c, 1.0, 0.0, std::f64::consts::PI / 3.0, 0.0, 2.0)
    };
    ($c:expr_2021, $i:expr_2021) => {
        $crate::scene::spot_light($c, $i, 0.0, std::f64::consts::PI / 3.0, 0.0, 2.0)
    };
    ($c:expr_2021, $i:expr_2021, $d:expr_2021) => {
        $crate::scene::spot_light($c, $i, $d, std::f64::consts::PI / 3.0, 0.0, 2.0)
    };
    ($c:expr_2021, $i:expr_2021, $d:expr_2021, $a:expr_2021) => {
        $crate::scene::spot_light($c, $i, $d, $a, 0.0, 2.0)
    };
    ($c:expr_2021, $i:expr_2021, $d:expr_2021, $a:expr_2021, $p:expr_2021) => {
        $crate::scene::spot_light($c, $i, $d, $a, $p, 2.0)
    };
    ($c:expr_2021, $i:expr_2021, $d:expr_2021, $a:expr_2021, $p:expr_2021, $k:expr_2021) => {
        $crate::scene::spot_light($c, $i, $d, $a, $p, $k)
    };
}
