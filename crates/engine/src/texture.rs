#![expect(
    non_upper_case_globals,
    reason = "the wraps, filters and colour spaces read as names (`RepeatWrapping`, `LinearFilter`) where every placer sets up its textures"
)]
//! Textures: shared pixel images (canvas-drawn or raw data) plus per-texture
//! sampling state (wrap, filters, mipmaps, uv offset / repeat / rotation, y-flip).

use crate::dmath::DMath;
use crate::math::*;
use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Wrap {
    #[default]
    Clamp,
    Repeat,
    Mirror,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ColorSpace {
    #[default]
    NoColorSpace,
    Srgb,
    Linear,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Filter {
    Nearest,
    #[default]
    Linear,
    NearestMipmapNearest,
    LinearMipmapLinear,
    LinearMipmapNearest,
    NearestMipmapLinear,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum PixelFormat {
    #[default]
    Rgba8,
    R8,
}

pub const RepeatWrapping: Wrap = Wrap::Repeat;
pub const ClampToEdgeWrapping: Wrap = Wrap::Clamp;
pub const MirroredRepeatWrapping: Wrap = Wrap::Mirror;
pub const SRGBColorSpace: ColorSpace = ColorSpace::Srgb;
pub const LinearSRGBColorSpace: ColorSpace = ColorSpace::Linear;
pub const NoColorSpace: ColorSpace = ColorSpace::NoColorSpace;
pub const LinearFilter: Filter = Filter::Linear;
pub const NearestFilter: Filter = Filter::Nearest;
pub const LinearMipmapLinearFilter: Filter = Filter::LinearMipmapLinear;

/// Pixel storage shared by a texture and its clones. RGBA8 with straight alpha
/// (uploaded as-is unless `premultiply_alpha` is set), rows top-down.
#[derive(Clone, Debug, Default)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub format: PixelFormat,
    pub data: Vec<u8>,
    pub version: u64,
    pub id: u64,
}
impl Image {
    pub fn new_rgba(width: u32, height: u32, data: Vec<u8>) -> Image {
        Image { width, height, format: PixelFormat::Rgba8, data, version: 1, id: crate::geometry::next_id() }
    }
    pub fn new_r8(width: u32, height: u32, data: Vec<u8>) -> Image {
        Image { width, height, format: PixelFormat::R8, data, version: 1, id: crate::geometry::next_id() }
    }
}

#[derive(Clone, Debug)]
pub struct Texture {
    pub id: u64,
    pub image: Rc<RefCell<Image>>,
    pub wrap_s: Wrap,
    pub wrap_t: Wrap,
    pub repeat: Vec2,
    pub offset: Vec2,
    pub center: Vec2,
    pub rotation: f64,
    pub flip_y: bool,
    pub color_space: ColorSpace,
    pub mag_filter: Filter,
    pub min_filter: Filter,
    pub generate_mipmaps: bool,
    pub anisotropy: u32,
    /// bumped by `needs_update()` (re-upload pixels)
    pub version: u64,
    pub name: String,
    /// premultiply the colour by alpha on upload
    pub premultiply_alpha: bool,
}
impl Texture {
    pub fn from_image(img: Image) -> Texture {
        Texture {
            id: crate::geometry::next_id(),
            image: Rc::new(RefCell::new(img)),
            wrap_s: Wrap::Clamp,
            wrap_t: Wrap::Clamp,
            repeat: Vec2::new(1.0, 1.0),
            offset: Vec2::default(),
            center: Vec2::default(),
            rotation: 0.0,
            flip_y: true,
            color_space: ColorSpace::NoColorSpace,
            mag_filter: Filter::Linear,
            min_filter: Filter::LinearMipmapLinear,
            generate_mipmaps: true,
            anisotropy: 1,
            version: 1,
            name: String::new(),
            premultiply_alpha: false,
        }
    }
    /// The uv transform (offset, repeat, rotation about `center`) as a 3x3
    /// column-major matrix.
    pub fn uv_transform(&self) -> [f64; 9] {
        let (tx, ty, sx, sy, rot, cx, cy) =
            (self.offset.x, self.offset.y, self.repeat.x, self.repeat.y, self.rotation, self.center.x, self.center.y);
        let c = rot.dcos();
        let s = rot.dsin();
        // row-major:
        // [sx*c, sx*s, -sx*(c*cx + s*cy) + cx + tx]
        // [-sy*s, sy*c, -sy*(-s*cx + c*cy) + cy + ty]
        // [0, 0, 1]
        [sx * c, -sy * s, 0.0, sx * s, sy * c, 0.0, -sx * (c * cx + s * cy) + cx + tx, -sy * (-s * cx + c * cy) + cy + ty, 1.0]
    }
}

/// Shared texture handle.
#[derive(Clone, Debug)]
pub struct Tex(pub Rc<RefCell<Texture>>);
impl PartialEq for Tex {
    fn eq(&self, o: &Tex) -> bool {
        Rc::ptr_eq(&self.0, &o.0)
    }
}
impl Tex {
    pub fn from_texture(t: Texture) -> Tex {
        Tex(Rc::new(RefCell::new(t)))
    }
    pub fn from_image(img: Image) -> Tex {
        Tex::from_texture(Texture::from_image(img))
    }
    /// Single-channel data texture (no y-flip, no mipmaps, nearest filtering)
    pub fn data_r8(data: Vec<u8>, w: u32, h: u32) -> Tex {
        let mut t = Texture::from_image(Image::new_r8(w, h, data));
        t.flip_y = false;
        t.generate_mipmaps = false;
        t.min_filter = Filter::Nearest;
        t.mag_filter = Filter::Nearest;
        Tex::from_texture(t)
    }
    /// RGBA data texture (no y-flip, no mipmaps, nearest filtering)
    pub fn data_rgba(data: Vec<u8>, w: u32, h: u32) -> Tex {
        let mut t = Texture::from_image(Image::new_rgba(w, h, data));
        t.flip_y = false;
        t.generate_mipmaps = false;
        t.min_filter = Filter::Nearest;
        t.mag_filter = Filter::Nearest;
        Tex::from_texture(t)
    }
    pub fn t(&self) -> RefMut<'_, Texture> {
        self.0.borrow_mut()
    }
    pub fn r(&self) -> Ref<'_, Texture> {
        self.0.borrow()
    }
    pub fn id(&self) -> u64 {
        self.0.borrow().id
    }
    pub fn image(&self) -> Rc<RefCell<Image>> {
        self.0.borrow().image.clone()
    }
    pub fn width(&self) -> u32 {
        self.0.borrow().image.borrow().width
    }
    pub fn height(&self) -> u32 {
        self.0.borrow().image.borrow().height
    }
    /// A new texture object (own sampling state) sharing the image.
    pub fn duplicate(&self) -> Tex {
        let mut t = self.0.borrow().clone();
        t.id = crate::geometry::next_id();
        Tex::from_texture(t)
    }
    /// Mark the pixels changed (re-uploaded before the next draw).
    pub fn needs_update(&self) {
        let t = self.0.borrow();
        t.image.borrow_mut().version += 1;
    }
    pub fn set_repeat_wrapping(&self) -> &Self {
        let mut t = self.t();
        t.wrap_s = Wrap::Repeat;
        t.wrap_t = Wrap::Repeat;
        drop(t);
        self
    }
    pub fn offset(&self) -> Vec2 {
        self.0.borrow().offset
    }
    pub fn set_offset(&self, x: impl Num, y: impl Num) {
        self.t().offset = Vec2::new(x.f(), y.f());
    }
    pub fn set_repeat(&self, x: impl Num, y: impl Num) {
        self.t().repeat = Vec2::new(x.f(), y.f());
    }
    pub fn dispose(&self) {}
}

/// Optional texture argument (`map: tex`, `map: &tex`, `map: None`).
pub trait IntoTexOpt {
    fn into_tex(self) -> Option<Tex>;
}
impl IntoTexOpt for Tex {
    fn into_tex(self) -> Option<Tex> {
        Some(self)
    }
}
impl IntoTexOpt for &Tex {
    fn into_tex(self) -> Option<Tex> {
        Some(self.clone())
    }
}
impl IntoTexOpt for Option<Tex> {
    fn into_tex(self) -> Option<Tex> {
        self
    }
}
impl IntoTexOpt for &Option<Tex> {
    fn into_tex(self) -> Option<Tex> {
        self.clone()
    }
}
