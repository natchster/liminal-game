//! Materials (basic, Lambert and Phong surfaces, points, sprites, lines) and
//! the game's custom shader patches, as data the Metal renderer interprets.
#![allow(non_snake_case, non_upper_case_globals)]

use crate::math::*;
use crate::texture::*;
use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum MatKind {
    Basic,
    #[default]
    Lambert,
    Phong,
    Standard,
    Points,
    Sprite,
    LineBasic,
    Depth,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Side {
    #[default]
    Front,
    Back,
    Double,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Blending {
    None,
    #[default]
    Normal,
    Additive,
    Subtractive,
    Multiply,
}

pub const FrontSide: Side = Side::Front;
pub const BackSide: Side = Side::Back;
pub const DoubleSide: Side = Side::Double;
pub const NoBlending: Blending = Blending::None;
pub const NormalBlending: Blending = Blending::Normal;
pub const AdditiveBlending: Blending = Blending::Additive;
pub const SubtractiveBlending: Blending = Blending::Subtractive;
pub const MultiplyBlending: Blending = Blending::Multiply;

/// Shader patches the game installs on a material (variants of the uber shader).
#[derive(Clone, Debug, PartialEq, Default)]
pub enum Patch {
    #[default]
    None,
    /// `world_uv_material`: world-space UVs for instanced architecture boxes
    ArchUV { scale_x: f64, scale_y: f64, wallpaper: bool },
    /// `with_skin_detail`: triplanar bump + varying shine from a tiling height map
    SkinDetail { map: Tex, scale: f64, strength: f64, shine: f64, rest_space: bool },
    /// the Smiler's emissive eye orbs
    SmilerEye,
    /// the Watcher's sheet (skin detail + soft shoulder + floor emissive)
    WatcherSheet { map: Tex, scale: f64, strength: f64 },
    /// the House shadows (boil, wisps, soft silhouette); `edge` = the rim skin
    Shadow { edge: bool },
    /// the Stairwell's own patch (see stairwell module)
    Stairwell { kind: u32 },
    /// a generic hook: renderer looks it up by name
    Named(String),
}

/// Per-frame uniforms some patches read (set by the game each frame).
#[derive(Clone, Debug, Default)]
pub struct PatchUniforms {
    pub f: [f32; 16],
}

#[derive(Clone, Debug)]
pub struct Material {
    pub id: u64,
    pub kind: MatKind,
    pub name: String,
    pub color: Color,
    pub emissive: Color,
    pub emissive_intensity: f64,
    pub specular: Color,
    pub shininess: f64,
    pub map: Option<Tex>,
    pub alpha_map: Option<Tex>,
    pub bump_map: Option<Tex>,
    pub bump_scale: f64,
    pub emissive_map: Option<Tex>,
    pub opacity: f64,
    pub transparent: bool,
    pub alpha_test: f64,
    pub side: Side,
    pub depth_write: bool,
    pub depth_test: bool,
    pub color_write: bool,
    pub blending: Blending,
    pub vertex_colors: bool,
    pub fog: bool,
    pub tone_mapped: bool,
    pub flat_shading: bool,
    pub wireframe: bool,
    pub polygon_offset: bool,
    pub polygon_offset_factor: f64,
    pub polygon_offset_units: f64,
    pub visible: bool,
    /// Point size and size attenuation (points); rotation (sprites)
    pub size: f64,
    pub size_attenuation: bool,
    pub rotation: f64,
    pub line_width: f64,
    pub patch: Patch,
    pub patch_uniforms: Rc<RefCell<PatchUniforms>>,
    /// Lit surfaces: value-noise mottling of the colour and grime near the
    /// floor, so flat paint reads as a used material (0 = none, 1 = full); a
    /// textured one also has its most vivid colours reined in
    pub wear: f64,
    /// Free for the game: marks a material whose look it has already settled
    pub look_settled: bool,
    /// bumped by `needs_update`
    pub version: u64,
}

impl Material {
    pub fn new(kind: MatKind) -> Material {
        Material {
            id: crate::geometry::next_id(),
            kind,
            name: String::new(),
            color: Color::white(),
            emissive: Color::default(),
            emissive_intensity: 1.0,
            specular: Color::hex(0x111111),
            shininess: 30.0,
            map: None,
            alpha_map: None,
            bump_map: None,
            bump_scale: 1.0,
            emissive_map: None,
            opacity: 1.0,
            transparent: false,
            alpha_test: 0.0,
            side: Side::Front,
            depth_write: true,
            depth_test: true,
            color_write: true,
            blending: Blending::Normal,
            vertex_colors: false,
            fog: true,
            tone_mapped: true,
            flat_shading: false,
            wireframe: false,
            polygon_offset: false,
            polygon_offset_factor: 0.0,
            polygon_offset_units: 0.0,
            visible: true,
            size: 1.0,
            size_attenuation: true,
            rotation: 0.0,
            line_width: 1.0,
            patch: Patch::None,
            patch_uniforms: Rc::new(RefCell::new(PatchUniforms::default())),
            wear: 0.0,
            look_settled: false,
            version: 1,
        }
    }
    /// does this material draw in the transparent pass
    pub fn is_transparent(&self) -> bool {
        self.transparent
    }
}

/// Shared material handle.
#[derive(Clone, Debug)]
pub struct Mat(pub Rc<RefCell<Material>>);
impl PartialEq for Mat {
    fn eq(&self, o: &Mat) -> bool {
        Rc::ptr_eq(&self.0, &o.0)
    }
}

impl Mat {
    pub fn new(kind: MatKind) -> Mat {
        let mut m = Material::new(kind);
        if kind == MatKind::Sprite {
            m.transparent = true;
        }
        if kind == MatKind::Points || kind == MatKind::Sprite || kind == MatKind::Basic || kind == MatKind::LineBasic {
            m.fog = true;
        }
        Mat(Rc::new(RefCell::new(m)))
    }
    pub fn basic() -> Mat {
        Mat::new(MatKind::Basic)
    }
    pub fn lambert() -> Mat {
        Mat::new(MatKind::Lambert)
    }
    pub fn phong() -> Mat {
        Mat::new(MatKind::Phong)
    }
    pub fn points_mat() -> Mat {
        Mat::new(MatKind::Points)
    }
    pub fn sprite_mat() -> Mat {
        Mat::new(MatKind::Sprite)
    }
    pub fn line_mat() -> Mat {
        Mat::new(MatKind::LineBasic)
    }
    /// Mutable access: `mat.m().opacity = 0.5`. Do not hold it across another `mat.m()`.
    pub fn m(&self) -> RefMut<'_, Material> {
        self.0.borrow_mut()
    }
    pub fn r(&self) -> Ref<'_, Material> {
        self.0.borrow()
    }
    pub fn id(&self) -> u64 {
        self.0.borrow().id
    }
    pub fn ptr_eq(&self, o: &Mat) -> bool {
        Rc::ptr_eq(&self.0, &o.0)
    }
    /// A new material with the same settings (and its own patch uniforms).
    pub fn duplicate(&self) -> Mat {
        let mut m = self.0.borrow().clone();
        m.id = crate::geometry::next_id();
        let pu = m.patch_uniforms.borrow().clone();
        m.patch_uniforms = Rc::new(RefCell::new(pu));
        Mat(Rc::new(RefCell::new(m)))
    }
    /// Mark the material changed (pipelines and uniforms are rebuilt).
    pub fn needs_update(&self) {
        self.m().version += 1;
    }
    // quick getters (copy out, safe inside expressions that also mutate)
    pub fn opacity_v(&self) -> f64 {
        self.0.borrow().opacity
    }
    pub fn color_v(&self) -> Color {
        self.0.borrow().color
    }
    pub fn emissive_v(&self) -> Color {
        self.0.borrow().emissive
    }
    pub fn emissive_intensity_v(&self) -> f64 {
        self.0.borrow().emissive_intensity
    }
    pub fn map_v(&self) -> Option<Tex> {
        self.0.borrow().map.clone()
    }
    pub fn set_opacity(&self, v: impl Num) {
        self.m().opacity = v.f();
    }
    pub fn set_color(&self, c: impl IntoColor) {
        self.m().color = c.into_color();
    }
    pub fn set_emissive(&self, c: impl IntoColor) {
        self.m().emissive = c.into_color();
    }
    pub fn set_emissive_intensity(&self, v: impl Num) {
        self.m().emissive_intensity = v.f();
    }
    pub fn set_map(&self, t: impl IntoTexOpt) {
        self.m().map = t.into_tex();
    }

    // ---- builder methods named after the material parameter keys the macros
    // accept (camelCase on purpose, hence `non_snake_case`)
    pub fn color(&self, c: impl IntoColor) -> &Self {
        self.m().color = c.into_color();
        self
    }
    pub fn emissive(&self, c: impl IntoColor) -> &Self {
        self.m().emissive = c.into_color();
        self
    }
    pub fn emissiveIntensity(&self, v: impl Num) -> &Self {
        self.m().emissive_intensity = v.f();
        self
    }
    pub fn specular(&self, c: impl IntoColor) -> &Self {
        self.m().specular = c.into_color();
        self
    }
    pub fn shininess(&self, v: impl Num) -> &Self {
        self.m().shininess = v.f();
        self
    }
    pub fn map(&self, t: impl IntoTexOpt) -> &Self {
        self.m().map = t.into_tex();
        self
    }
    pub fn alphaMap(&self, t: impl IntoTexOpt) -> &Self {
        self.m().alpha_map = t.into_tex();
        self
    }
    pub fn bumpMap(&self, t: impl IntoTexOpt) -> &Self {
        self.m().bump_map = t.into_tex();
        self
    }
    pub fn bumpScale(&self, v: impl Num) -> &Self {
        self.m().bump_scale = v.f();
        self
    }
    pub fn emissiveMap(&self, t: impl IntoTexOpt) -> &Self {
        self.m().emissive_map = t.into_tex();
        self
    }
    pub fn opacity(&self, v: impl Num) -> &Self {
        self.m().opacity = v.f();
        self
    }
    pub fn transparent(&self, v: bool) -> &Self {
        self.m().transparent = v;
        self
    }
    pub fn alphaTest(&self, v: impl Num) -> &Self {
        self.m().alpha_test = v.f();
        self
    }
    pub fn side(&self, v: Side) -> &Self {
        self.m().side = v;
        self
    }
    pub fn depthWrite(&self, v: bool) -> &Self {
        self.m().depth_write = v;
        self
    }
    pub fn depthTest(&self, v: bool) -> &Self {
        self.m().depth_test = v;
        self
    }
    pub fn colorWrite(&self, v: bool) -> &Self {
        self.m().color_write = v;
        self
    }
    pub fn blending(&self, v: Blending) -> &Self {
        self.m().blending = v;
        self
    }
    pub fn vertexColors(&self, v: bool) -> &Self {
        self.m().vertex_colors = v;
        self
    }
    pub fn fog(&self, v: bool) -> &Self {
        self.m().fog = v;
        self
    }
    pub fn toneMapped(&self, v: bool) -> &Self {
        self.m().tone_mapped = v;
        self
    }
    pub fn flatShading(&self, v: bool) -> &Self {
        self.m().flat_shading = v;
        self
    }
    pub fn wireframe(&self, v: bool) -> &Self {
        self.m().wireframe = v;
        self
    }
    pub fn polygonOffset(&self, v: bool) -> &Self {
        self.m().polygon_offset = v;
        self
    }
    pub fn polygonOffsetFactor(&self, v: impl Num) -> &Self {
        self.m().polygon_offset_factor = v.f();
        self
    }
    pub fn polygonOffsetUnits(&self, v: impl Num) -> &Self {
        self.m().polygon_offset_units = v.f();
        self
    }
    pub fn visible(&self, v: bool) -> &Self {
        self.m().visible = v;
        self
    }
    pub fn size(&self, v: impl Num) -> &Self {
        self.m().size = v.f();
        self
    }
    pub fn sizeAttenuation(&self, v: bool) -> &Self {
        self.m().size_attenuation = v;
        self
    }
    pub fn rotation(&self, v: impl Num) -> &Self {
        self.m().rotation = v.f();
        self
    }
    pub fn linewidth(&self, v: impl Num) -> &Self {
        self.m().line_width = v.f();
        self
    }
    pub fn name(&self, v: &str) -> &Self {
        self.m().name = v.to_string();
        self
    }
    // PBR/legacy keys accepted and ignored (Lambert/Phong have no use for them)
    pub fn roughness(&self, _v: impl Num) -> &Self {
        self
    }
    pub fn metalness(&self, _v: impl Num) -> &Self {
        self
    }
    pub fn reflectivity(&self, _v: impl Num) -> &Self {
        self
    }
    pub fn dithering(&self, _v: bool) -> &Self {
        self
    }
    pub fn premultipliedAlpha(&self, _v: bool) -> &Self {
        self
    }
}

/// Build a material from a parameter list:
/// `lambert!({ color: 0x8a6a40, transparent: true, opacity: 0.5 })`.
#[macro_export]
macro_rules! lambert {
    () => { $crate::material::Mat::lambert() };
    ({ $($k:ident : $v:expr),* $(,)? }) => {{ let m = $crate::material::Mat::lambert(); $( m.$k($v); )* m }};
}
#[macro_export]
macro_rules! basic_mat {
    () => { $crate::material::Mat::basic() };
    ({ $($k:ident : $v:expr),* $(,)? }) => {{ let m = $crate::material::Mat::basic(); $( m.$k($v); )* m }};
}
#[macro_export]
macro_rules! phong {
    () => { $crate::material::Mat::phong() };
    ({ $($k:ident : $v:expr),* $(,)? }) => {{ let m = $crate::material::Mat::phong(); $( m.$k($v); )* m }};
}
/// Standard (PBR) materials are drawn with the Lambert model; PBR keys are ignored.
#[macro_export]
macro_rules! standard_mat {
    () => { $crate::material::Mat::lambert() };
    ({ $($k:ident : $v:expr),* $(,)? }) => {{ let m = $crate::material::Mat::lambert(); $( m.$k($v); )* m }};
}
#[macro_export]
macro_rules! points_mat {
    ({ $($k:ident : $v:expr),* $(,)? }) => {{ let m = $crate::material::Mat::points_mat(); $( m.$k($v); )* m }};
}
#[macro_export]
macro_rules! sprite_mat {
    () => { $crate::material::Mat::sprite_mat() };
    ({ $($k:ident : $v:expr),* $(,)? }) => {{ let m = $crate::material::Mat::sprite_mat(); $( m.$k($v); )* m }};
}
#[macro_export]
macro_rules! line_mat {
    () => { $crate::material::Mat::line_mat() };
    ({ $($k:ident : $v:expr),* $(,)? }) => {{ let m = $crate::material::Mat::line_mat(); $( m.$k($v); )* m }};
}

/// Material argument for meshes: one material or a per-group array.
pub trait IntoMats {
    fn into_mats(self) -> Vec<Mat>;
}
impl IntoMats for Mat {
    fn into_mats(self) -> Vec<Mat> {
        vec![self]
    }
}
impl IntoMats for &Mat {
    fn into_mats(self) -> Vec<Mat> {
        vec![self.clone()]
    }
}
impl IntoMats for Vec<Mat> {
    fn into_mats(self) -> Vec<Mat> {
        self
    }
}
impl IntoMats for &Vec<Mat> {
    fn into_mats(self) -> Vec<Mat> {
        self.clone()
    }
}
impl IntoMats for &[Mat] {
    fn into_mats(self) -> Vec<Mat> {
        self.to_vec()
    }
}
impl<const N: usize> IntoMats for [Mat; N] {
    fn into_mats(self) -> Vec<Mat> {
        self.to_vec()
    }
}
impl<const N: usize> IntoMats for &[Mat; N] {
    fn into_mats(self) -> Vec<Mat> {
        self.to_vec()
    }
}
