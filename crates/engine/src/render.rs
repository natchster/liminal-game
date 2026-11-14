//! The Metal renderer: frustum-culls the scene graph, sorts opaque items front
//! to back and transparent ones back to front, shades them per fragment (basic /
//! Lambert / Phong with point, spot, hemisphere, ambient and directional lights
//! and exponential-squared fog), then composites the HUD and the CSS-style
//! overlay layers.
#![allow(clippy::too_many_arguments)]

use crate::geometry::Geometry;
use crate::material::*;
use crate::math::*;
use crate::scene::*;
use crate::texture::*;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_foundation::NSString;
use objc2_metal::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::c_void;
use std::ptr::NonNull;
use std::rc::{Rc, Weak};

type Dev = ProtocolObject<dyn MTLDevice>;
type Buf = Retained<ProtocolObject<dyn MTLBuffer>>;
type MTex = Retained<ProtocolObject<dyn MTLTexture>>;

const SHADER_SRC: &str = include_str!("shaders.metal");
const COMPOSITE_SRC: &str = include_str!("composite.metal");
pub const SAMPLE_COUNT: usize = 4;
const FRAME_ARENA: usize = 24 << 20;

/// The light-map patch's uniforms (`LL`).
#[derive(Clone, Debug, Default)]
pub struct LocalLight {
    pub on: bool,
    pub map: Option<Tex>,
    pub grid: [f64; 4],
    pub scale: f64,
    pub amb_d: Vec3,
    pub sky_d: Vec3,
    pub gnd_d: Vec3,
    pub fog_lit: Color,
    pub fog_ratio: f64,
}

/// Scene-wide render state (background, fog, ...).
#[derive(Clone, Debug)]
pub struct RenderGlobals {
    pub background: Color,
    /// exponential-squared (FogExp2-style) fog: colour and density
    pub fog: Option<(Color, f64)>,
    pub ll: LocalLight,
    /// render resolution relative to the window's physical pixels (quality governor)
    pub pixel_ratio: f64,
    /// CSS `filter` on the 3D canvas: up to four affine colour matrices (3×4,
    /// row-major: `rgb' = M · [r g b 1]`), applied in order to the displayed
    /// (sRGB-encoded) colour, each result clamped to 0..1 as CSS does
    pub scene_filter: Vec<[f32; 12]>,
    /// how much lit surfaces lose of their indirect light toward the floor
    /// under them (0 = none, 1 = full; off where the light map does not
    /// hold the floors): the light map's alpha, the floor's height as
    /// `floor_base` + code × `floor_step` (0 where there is none, the void:
    /// no floor under it). A thing that takes wear by use stands on its own
    /// (`UseWear::floor`), or where the map's floors have no gaps (the
    /// rooms) on the map's where that is lower (a pool's)
    pub floor_occlusion: f64,
    pub floor_base: f64,
    pub floor_step: f64,
    /// some places of the light map have no floor (the Building's void and
    /// masses): its blue holds whether each has one, not the ceiling, and
    /// the floor of a sample is its alpha over its blue (the part of it
    /// that has one; corner occlusion is off)
    pub floor_gaps: bool,
    /// how much indirect light a surface loses where walls meet (from the
    /// light map's G and B; 0 = none, 1 = full; off where the map does not
    /// hold the place's shape)
    pub corner_occlusion: f64,
}
impl Default for RenderGlobals {
    fn default() -> Self {
        RenderGlobals {
            background: Color::default(),
            fog: None,
            ll: LocalLight::default(),
            pixel_ratio: 1.0,
            scene_filter: vec![],
            floor_occlusion: 0.0,
            floor_base: 0.0,
            floor_step: 0.0,
            floor_gaps: false,
            corner_occlusion: 0.0,
        }
    }
}

pub use crate::css::filter as css_filter;

/// Per-frame statistics (draw calls, primitives, lights, compiled programs).
#[derive(Clone, Copy, Debug, Default)]
pub struct RenderInfo {
    pub calls: u32,
    pub triangles: u64,
    pub points: u64,
    pub lines: u64,
    pub lights: u32,
    pub programs: u32,
    pub frame_ms: f64,
}

/// How an overlay layer combines with what is under it (CSS mix-blend-mode).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OBlend {
    Normal = 0,
    Multiply = 1,
    Screen = 2,
    Add = 3,
    Overlay = 4,
    SoftLight = 5,
    ColorDodge = 6,
    Darken = 7,
    Lighten = 8,
}

/// A premultiplied RGBA image to composite (the HUD canvas, the grain).
pub struct OverlayImage<'a> {
    pub id: u64,
    pub version: u64,
    pub width: u32,
    pub height: u32,
    pub premul_rgba: &'a [u8],
}

/// One overlay layer, drawn in order over the 3D view. Rects are in 0..1
/// screen units (y down). Colours are straight alpha, sRGB.
pub enum OverlayOp<'a> {
    Image {
        image: OverlayImage<'a>,
        rect: [f32; 4],
        opacity: f32,
        blend: OBlend,
        tile: [f32; 2],
        offset: [f32; 2],
        smooth: bool,
    },
    Fill {
        color: [f32; 4],
        blend: OBlend,
    },
    /// CSS radial-gradient: `center` in 0..1, `radius` (rx, ry) in 0..1 of the
    /// width / height, up to 4 stops (t, rgba).
    Radial {
        center: [f32; 2],
        radius: [f32; 2],
        stops: Vec<(f32, [f32; 4])>,
        blend: OBlend,
        opacity: f32,
    },
    /// The scene shifted per channel (chromatic aberration), px offsets.
    Aberration {
        offset_px: f32,
        opacity: f32,
    },
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct FrameU {
    view: [f32; 16],
    proj: [f32; 16],
    ambient: [f32; 4],
    fog: [f32; 4],
    ll_grid: [f32; 4],
    ll_amb: [f32; 4],
    ll_sky: [f32; 4],
    ll_gnd: [f32; 4],
    ll_fog_lit: [f32; 4],
    ll_params: [f32; 4],
    viewport: [f32; 4],
    counts: [u32; 4],
    look: [f32; 4],
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct HemiU {
    sky: [f32; 4],
    ground: [f32; 4],
    dir: [f32; 4],
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct PointU {
    pos: [f32; 4],
    color: [f32; 4],
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct SpotU {
    pos: [f32; 4],
    dir: [f32; 4],
    color: [f32; 4],
    cone: [f32; 4],
}
#[repr(C)]
#[derive(Clone, Copy)]
struct ObjectU {
    model: [f32; 16],
    model_view: [f32; 16],
    normal: [f32; 16],
    bind: [f32; 16],
    bind_inv: [f32; 16],
    morph: [f32; 8],
    sprite: [f32; 4],
    /// the thing's wear by use (`UseWear`): x handled, y chips, z its
    /// floor's height; w 1 when each instance has its own (buffer 15)
    use_wear: [f32; 4],
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct MaterialU {
    diffuse: [f32; 4],
    emissive: [f32; 4],
    specular: [f32; 4],
    map_t0: [f32; 4],
    map_t1: [f32; 4],
    misc: [f32; 4],
    arch: [f32; 4],
    patch: [f32; 16],
    surf: [f32; 4],
    portal: [f32; 16],
}
#[repr(C)]
#[derive(Clone, Copy)]
struct LightListU {
    count: u32,
    pad: [u32; 3],
    idx: [u16; 96],
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct VariantKey {
    kind: i32,
    map: bool,
    vcolor: i32,
    instanced: bool,
    inst_color: bool,
    skinned: bool,
    morphs: i32,
    fog: bool,
    alpha_test: bool,
    double: bool,
    flip: bool,
    flat: bool,
    ll: bool,
    patch: i32,
    normal: bool,
    uv: bool,
    opaque: bool,
    bump: bool,
    morph_nrm: bool,
    morph_rel: bool,
    size_atten: bool,
    vc_data: bool,
    wear: bool,
    portal: bool,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct PipeKey {
    v: VariantKey,
    blend: u8,
    color_write: bool,
}

struct GpuGeo {
    weak: Weak<RefCell<Geometry>>,
    version: u64,
    position: Option<Buf>,
    normal: Option<Buf>,
    uv: Option<Buf>,
    color: Option<Buf>,
    skin_idx: Option<Buf>,
    skin_w: Option<Buf>,
    morph_pos: Option<Buf>,
    morph_nrm: Option<Buf>,
    morphs: usize,
    index: Option<Buf>,
    vcount: usize,
    icount: usize,
}
struct GpuTex {
    weak: Weak<RefCell<Image>>,
    version: u64,
    tex: MTex,
    srgb: bool,
}
struct GpuInst {
    weak: Weak<RefCell<Node>>,
    version: u64,
    mats: Buf,
    cols: Option<Buf>,
    /// each instance's wear by use (`InstanceData::use_wear`)
    wear: Option<Buf>,
}

struct Arena {
    buf: Buf,
    off: usize,
    cb: Option<Retained<ProtocolObject<dyn MTLCommandBuffer>>>,
}

/// One renderable of a frame's draw list.
struct Item {
    obj: Obj,
    geo: Option<crate::geometry::Geo>,
    mat: Mat,
    group: Option<(u32, u32)>,
    group_order: i32,
    render_order: i32,
    z: f64,
    id: u64,
    mat_id: u64,
    transparent: bool,
    world: Mat4,
    sphere: Sphere,
}

pub struct Renderer {
    pub device: Retained<Dev>,
    queue: Retained<ProtocolObject<dyn MTLCommandQueue>>,
    lib: Retained<ProtocolObject<dyn MTLLibrary>>,
    comp_lib: Retained<ProtocolObject<dyn MTLLibrary>>,
    pipes: HashMap<PipeKey, Retained<ProtocolObject<dyn MTLRenderPipelineState>>>,
    comp_pipes: HashMap<u32, Retained<ProtocolObject<dyn MTLRenderPipelineState>>>,
    depth_states: HashMap<(bool, bool), Retained<ProtocolObject<dyn MTLDepthStencilState>>>,
    samplers: HashMap<(u8, u8, u8, u8, u8, u8), Retained<ProtocolObject<dyn MTLSamplerState>>>,
    geos: HashMap<u64, GpuGeo>,
    texs: HashMap<u64, GpuTex>,
    insts: HashMap<u64, GpuInst>,
    overlay_texs: HashMap<u64, (u64, MTex, u32, u32)>,
    arenas: Vec<Arena>,
    arena_i: usize,
    msaa_color: Option<MTex>,
    msaa_depth: Option<MTex>,
    scene_tex: Option<MTex>,
    rt_size: (usize, usize),
    dummy_buf: Buf,
    dummy_tex: MTex,
    white_tex: MTex,
    pub info: RenderInfo,
    frame_no: u64,
}

fn ns(s: &str) -> Retained<NSString> {
    NSString::from_str(s)
}
fn mat_f32(m: &Mat4) -> [f32; 16] {
    m.to_f32()
}

impl Renderer {
    pub fn new(device: Retained<Dev>) -> Renderer {
        let queue = device.newCommandQueue().expect("command queue");
        let opts = MTLCompileOptions::new();
        #[allow(deprecated)] // `setMathMode` needs macOS 15; the app runs on macOS 12+
        opts.setFastMathEnabled(false);
        let lib = device
            .newLibraryWithSource_options_error(&ns(SHADER_SRC), Some(&opts))
            .unwrap_or_else(|e| panic!("shader compile: {}", e.localizedDescription()));
        let comp_lib = device
            .newLibraryWithSource_options_error(&ns(COMPOSITE_SRC), Some(&opts))
            .unwrap_or_else(|e| panic!("composite shader: {}", e.localizedDescription()));
        let dummy_buf = new_buffer(&device, &[0u8; 256]);
        let dummy_tex = make_texture(&device, 1, 1, MTLPixelFormat::RGBA8Unorm, false, &[255, 255, 255, 255]);
        let white_tex = make_texture(&device, 1, 1, MTLPixelFormat::R8Unorm, false, &[255]);
        let arenas = (0..3)
            .map(|_| Arena {
                buf: device.newBufferWithLength_options(FRAME_ARENA, MTLResourceOptions::StorageModeShared).unwrap(),
                off: 0,
                cb: None,
            })
            .collect();
        Renderer {
            device,
            queue,
            lib,
            comp_lib,
            pipes: HashMap::new(),
            comp_pipes: HashMap::new(),
            depth_states: HashMap::new(),
            samplers: HashMap::new(),
            geos: HashMap::new(),
            texs: HashMap::new(),
            insts: HashMap::new(),
            overlay_texs: HashMap::new(),
            arenas,
            arena_i: 0,
            msaa_color: None,
            msaa_depth: None,
            scene_tex: None,
            rt_size: (0, 0),
            dummy_buf,
            dummy_tex,
            white_tex,
            info: RenderInfo::default(),
            frame_no: 0,
        }
    }

    // ------------------------------------------------------------ resources
    fn ensure_targets(&mut self, w: usize, h: usize) {
        if self.rt_size == (w, h) && self.scene_tex.is_some() {
            return;
        }
        let d = &self.device;
        unsafe {
            let cd =
                MTLTextureDescriptor::texture2DDescriptorWithPixelFormat_width_height_mipmapped(MTLPixelFormat::BGRA8Unorm, w, h, false);
            cd.setTextureType(MTLTextureType::Type2DMultisample);
            cd.setSampleCount(SAMPLE_COUNT);
            cd.setUsage(MTLTextureUsage::RenderTarget);
            cd.setStorageMode(MTLStorageMode::Memoryless);
            self.msaa_color = d.newTextureWithDescriptor(&cd);
            let dd =
                MTLTextureDescriptor::texture2DDescriptorWithPixelFormat_width_height_mipmapped(MTLPixelFormat::Depth32Float, w, h, false);
            dd.setTextureType(MTLTextureType::Type2DMultisample);
            dd.setSampleCount(SAMPLE_COUNT);
            dd.setUsage(MTLTextureUsage::RenderTarget);
            dd.setStorageMode(MTLStorageMode::Memoryless);
            self.msaa_depth = d.newTextureWithDescriptor(&dd);
            let sd =
                MTLTextureDescriptor::texture2DDescriptorWithPixelFormat_width_height_mipmapped(MTLPixelFormat::BGRA8Unorm, w, h, false);
            sd.setUsage(MTLTextureUsage::RenderTarget | MTLTextureUsage::ShaderRead);
            sd.setStorageMode(MTLStorageMode::Private);
            self.scene_tex = d.newTextureWithDescriptor(&sd);
        }
        self.rt_size = (w, h);
    }

    fn gpu_geo(&mut self, g: &crate::geometry::Geo) -> u64 {
        let (id, version) = {
            let gg = g.0.borrow();
            (gg.id, gg.version)
        };
        if let Some(e) = self.geos.get(&id)
            && e.version == version
        {
            return id;
        }
        let gg = g.0.borrow();
        let d = &self.device;
        let fbuf = |v: &Vec<f32>| if v.is_empty() { None } else { Some(new_buffer(d, bytemuck_f32(v))) };
        let vcount = gg.position.len() / 3;
        let (mut mp, mut mn) = (vec![], vec![]);
        for m in &gg.morph_position {
            mp.extend_from_slice(&m[..m.len().min(vcount * 3)]);
            if m.len() < vcount * 3 {
                mp.resize(mp.len() + vcount * 3 - m.len(), 0.0);
            }
        }
        for m in &gg.morph_normal {
            mn.extend_from_slice(&m[..m.len().min(vcount * 3)]);
            if m.len() < vcount * 3 {
                mn.resize(mn.len() + vcount * 3 - m.len(), 0.0);
            }
        }
        // colours with item size 4 stay 4; 3 stay 3
        let e = GpuGeo {
            weak: Rc::downgrade(&g.0),
            version,
            position: fbuf(&gg.position),
            normal: fbuf(&gg.normal),
            uv: fbuf(&gg.uv),
            color: fbuf(&gg.color),
            skin_idx: fbuf(&gg.skin_index),
            skin_w: fbuf(&gg.skin_weight),
            morph_pos: fbuf(&mp),
            morph_nrm: fbuf(&mn),
            morphs: gg.morph_position.len(),
            index: gg.index.as_ref().filter(|i| !i.is_empty()).map(|i| new_buffer(d, bytemuck_u32(i))),
            vcount,
            icount: gg.index.as_ref().map(|i| i.len()).unwrap_or(vcount),
        };
        drop(gg);
        self.geos.insert(id, e);
        id
    }

    fn gpu_tex(&mut self, t: &Tex) -> (u64, bool) {
        let tt = t.0.borrow();
        let img = tt.image.clone();
        let im = img.borrow();
        let id = im.id;
        let srgb = tt.color_space == ColorSpace::Srgb;
        if let Some(e) = self.texs.get(&id)
            && e.version == im.version
            && e.srgb == srgb
        {
            return (id, tt.flip_y);
        }
        let fmt = match im.format {
            PixelFormat::R8 => MTLPixelFormat::R8Unorm,
            PixelFormat::Rgba8 => {
                if srgb {
                    MTLPixelFormat::RGBA8Unorm_sRGB
                } else {
                    MTLPixelFormat::RGBA8Unorm
                }
            }
        };
        let mips = tt.generate_mipmaps
            && matches!(
                tt.min_filter,
                Filter::LinearMipmapLinear | Filter::LinearMipmapNearest | Filter::NearestMipmapLinear | Filter::NearestMipmapNearest
            );
        let w = im.width.max(1) as usize;
        let h = im.height.max(1) as usize;
        let reuse = self.texs.get(&id).filter(|e| e.tex.width() == w && e.tex.height() == h && e.srgb == srgb).map(|e| e.tex.clone());
        let tex = match reuse {
            Some(t) => {
                upload(&t, w, h, fmt, &im.data);
                t
            }
            None => make_texture(&self.device, w, h, fmt, mips, &im.data),
        };
        if mips && tex.mipmapLevelCount() > 1 {
            let cb = self.queue.commandBuffer().unwrap();
            let bl = cb.blitCommandEncoder().unwrap();
            bl.generateMipmapsForTexture(&tex);
            bl.endEncoding();
            cb.commit();
        }
        let flip = tt.flip_y;
        self.texs.insert(id, GpuTex { weak: Rc::downgrade(&img), version: im.version, tex, srgb });
        (id, flip)
    }

    fn sampler(&mut self, t: Option<&Texture>) -> Retained<ProtocolObject<dyn MTLSamplerState>> {
        let key = match t {
            Some(t) => (
                t.wrap_s as u8,
                t.wrap_t as u8,
                t.mag_filter as u8,
                t.min_filter as u8,
                (t.generate_mipmaps) as u8,
                t.anisotropy.clamp(1, 16) as u8,
            ),
            None => (0, 0, 1, 1, 0, 1),
        };
        if let Some(s) = self.samplers.get(&key) {
            return s.clone();
        }
        let d = MTLSamplerDescriptor::new();
        let wrap = |w: u8| match w {
            1 => MTLSamplerAddressMode::Repeat,
            2 => MTLSamplerAddressMode::MirrorRepeat,
            _ => MTLSamplerAddressMode::ClampToEdge,
        };
        d.setSAddressMode(wrap(key.0));
        d.setTAddressMode(wrap(key.1));
        let lin = |f: u8| {
            if f == Filter::Nearest as u8 || f == Filter::NearestMipmapNearest as u8 || f == Filter::NearestMipmapLinear as u8 {
                MTLSamplerMinMagFilter::Nearest
            } else {
                MTLSamplerMinMagFilter::Linear
            }
        };
        d.setMagFilter(lin(key.2));
        d.setMinFilter(lin(key.3));
        let mip = if key.4 == 0 {
            MTLSamplerMipFilter::NotMipmapped
        } else if key.3 == Filter::LinearMipmapLinear as u8 || key.3 == Filter::NearestMipmapLinear as u8 {
            MTLSamplerMipFilter::Linear
        } else if key.3 == Filter::LinearMipmapNearest as u8 || key.3 == Filter::NearestMipmapNearest as u8 {
            MTLSamplerMipFilter::Nearest
        } else {
            MTLSamplerMipFilter::NotMipmapped
        };
        d.setMipFilter(mip);
        if mip != MTLSamplerMipFilter::NotMipmapped {
            d.setMaxAnisotropy(key.5 as usize);
        }
        let s = self.device.newSamplerStateWithDescriptor(&d).unwrap();
        self.samplers.insert(key, s.clone());
        s
    }

    fn depth_state(&mut self, test: bool, write: bool) -> Retained<ProtocolObject<dyn MTLDepthStencilState>> {
        if let Some(s) = self.depth_states.get(&(test, write)) {
            return s.clone();
        }
        let d = MTLDepthStencilDescriptor::new();
        d.setDepthCompareFunction(if test { MTLCompareFunction::LessEqual } else { MTLCompareFunction::Always });
        d.setDepthWriteEnabled(write);
        let s = self.device.newDepthStencilStateWithDescriptor(&d).unwrap();
        self.depth_states.insert((test, write), s.clone());
        s
    }

    fn pipeline(&mut self, key: PipeKey) -> Retained<ProtocolObject<dyn MTLRenderPipelineState>> {
        if let Some(p) = self.pipes.get(&key) {
            return p.clone();
        }
        let v = key.v;
        let fc = MTLFunctionConstantValues::new();
        unsafe {
            let set_i = |i: usize, val: i32| {
                fc.setConstantValue_type_atIndex(NonNull::new(&val as *const i32 as *mut c_void).unwrap(), MTLDataType::Int, i)
            };
            let set_b = |i: usize, val: bool| {
                fc.setConstantValue_type_atIndex(NonNull::new(&val as *const bool as *mut c_void).unwrap(), MTLDataType::Bool, i)
            };
            set_i(0, v.kind);
            set_b(1, v.map);
            set_i(2, v.vcolor);
            set_b(3, v.instanced);
            set_b(4, v.inst_color);
            set_b(5, v.skinned);
            set_i(6, v.morphs);
            set_b(7, v.fog);
            set_b(8, v.alpha_test);
            set_b(9, v.double);
            set_b(10, v.flip);
            set_b(11, v.flat);
            set_b(12, v.ll);
            set_i(13, v.patch);
            set_b(14, v.normal);
            set_b(15, v.uv);
            set_b(16, v.opaque);
            set_b(17, v.bump);
            set_b(18, v.morph_nrm);
            set_b(19, v.morph_rel);
            set_b(20, v.size_atten);
            set_b(21, v.vc_data);
            set_b(22, v.wear);
            set_b(23, v.portal);
        }
        let vf = self
            .lib
            .newFunctionWithName_constantValues_error(&ns("vs_main"), &fc)
            .unwrap_or_else(|e| panic!("vs variant {:?}: {}", v, e.localizedDescription()));
        let ff = self
            .lib
            .newFunctionWithName_constantValues_error(&ns("fs_main"), &fc)
            .unwrap_or_else(|e| panic!("fs variant {:?}: {}", v, e.localizedDescription()));
        let pd = MTLRenderPipelineDescriptor::new();
        pd.setVertexFunction(Some(&vf));
        pd.setFragmentFunction(Some(&ff));
        pd.setRasterSampleCount(SAMPLE_COUNT);
        pd.setDepthAttachmentPixelFormat(MTLPixelFormat::Depth32Float);
        if v.kind == 3 {
            unsafe {
                pd.setInputPrimitiveTopology(MTLPrimitiveTopologyClass::Point);
            }
        }
        // lines keep the unspecified topology class: the uber vertex function always
        // writes [[point_size]], which Metal rejects for a declared Line class
        unsafe {
            let ca = pd.colorAttachments().objectAtIndexedSubscript(0);
            ca.setPixelFormat(MTLPixelFormat::BGRA8Unorm);
            if !key.color_write {
                ca.setWriteMask(MTLColorWriteMask::empty());
            }
            match key.blend {
                0 => ca.setBlendingEnabled(false),
                1 => {
                    // normal blending (straight alpha)
                    ca.setBlendingEnabled(true);
                    ca.setRgbBlendOperation(MTLBlendOperation::Add);
                    ca.setAlphaBlendOperation(MTLBlendOperation::Add);
                    ca.setSourceRGBBlendFactor(MTLBlendFactor::SourceAlpha);
                    ca.setDestinationRGBBlendFactor(MTLBlendFactor::OneMinusSourceAlpha);
                    ca.setSourceAlphaBlendFactor(MTLBlendFactor::One);
                    ca.setDestinationAlphaBlendFactor(MTLBlendFactor::OneMinusSourceAlpha);
                }
                2 => {
                    // additive blending
                    ca.setBlendingEnabled(true);
                    ca.setRgbBlendOperation(MTLBlendOperation::Add);
                    ca.setAlphaBlendOperation(MTLBlendOperation::Add);
                    ca.setSourceRGBBlendFactor(MTLBlendFactor::SourceAlpha);
                    ca.setDestinationRGBBlendFactor(MTLBlendFactor::One);
                    ca.setSourceAlphaBlendFactor(MTLBlendFactor::SourceAlpha);
                    ca.setDestinationAlphaBlendFactor(MTLBlendFactor::One);
                }
                3 => {
                    // subtractive blending
                    ca.setBlendingEnabled(true);
                    ca.setRgbBlendOperation(MTLBlendOperation::Add);
                    ca.setAlphaBlendOperation(MTLBlendOperation::Add);
                    ca.setSourceRGBBlendFactor(MTLBlendFactor::Zero);
                    ca.setDestinationRGBBlendFactor(MTLBlendFactor::OneMinusSourceColor);
                    ca.setSourceAlphaBlendFactor(MTLBlendFactor::Zero);
                    ca.setDestinationAlphaBlendFactor(MTLBlendFactor::One);
                }
                _ => {
                    // multiply blending
                    ca.setBlendingEnabled(true);
                    ca.setRgbBlendOperation(MTLBlendOperation::Add);
                    ca.setAlphaBlendOperation(MTLBlendOperation::Add);
                    ca.setSourceRGBBlendFactor(MTLBlendFactor::Zero);
                    ca.setDestinationRGBBlendFactor(MTLBlendFactor::SourceColor);
                    ca.setSourceAlphaBlendFactor(MTLBlendFactor::Zero);
                    ca.setDestinationAlphaBlendFactor(MTLBlendFactor::SourceAlpha);
                }
            }
        }
        let p = self
            .device
            .newRenderPipelineStateWithDescriptor_error(&pd)
            .unwrap_or_else(|e| panic!("pipeline {:?}: {}", v, e.localizedDescription()));
        self.pipes.insert(key, p.clone());
        self.info.programs = self.pipes.len() as u32;
        p
    }

    fn comp_pipeline(&mut self, which: u32) -> Retained<ProtocolObject<dyn MTLRenderPipelineState>> {
        if let Some(p) = self.comp_pipes.get(&which) {
            return p.clone();
        }
        let pd = MTLRenderPipelineDescriptor::new();
        let vf = self.comp_lib.newFunctionWithName(&ns("vs_quad")).unwrap();
        let fname = match which {
            0 => "fs_scene",
            1 => "fs_image",
            2 => "fs_fill",
            3 => "fs_radial",
            _ => "fs_aberration",
        };
        let ff = self.comp_lib.newFunctionWithName(&ns(fname)).unwrap();
        pd.setVertexFunction(Some(&vf));
        pd.setFragmentFunction(Some(&ff));
        unsafe {
            pd.colorAttachments().objectAtIndexedSubscript(0).setPixelFormat(MTLPixelFormat::BGRA8Unorm);
        }
        let p = self
            .device
            .newRenderPipelineStateWithDescriptor_error(&pd)
            .unwrap_or_else(|e| panic!("composite pipeline: {}", e.localizedDescription()));
        self.comp_pipes.insert(which, p.clone());
        p
    }

    /// Drop GPU copies of geometry / textures nothing references any more.
    fn sweep(&mut self) {
        self.geos.retain(|_, g| g.weak.strong_count() > 0);
        self.texs.retain(|_, t| t.weak.strong_count() > 0);
        self.insts.retain(|_, i| i.weak.strong_count() > 0);
    }

    fn arena_alloc(&mut self, bytes: &[u8]) -> (Buf, usize) {
        let a = &mut self.arenas[self.arena_i];
        let off = (a.off + 255) & !255;
        if off + bytes.len() > FRAME_ARENA {
            // overflow: a one-off buffer
            return (new_buffer(&self.device, bytes), 0);
        }
        unsafe {
            let p = a.buf.contents().as_ptr() as *mut u8;
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), p.add(off), bytes.len());
        }
        a.off = off + bytes.len();
        (a.buf.clone(), off)
    }

    /// Compile every pipeline the scene will need now, before its first frame.
    pub fn warm_up(&mut self, scene: &Obj, globals: &RenderGlobals) {
        let mut keys = vec![];
        let fog = globals.fog.is_some();
        scene.traverse(&mut |o| {
            let n = o.r();
            let (geo, mats, inst, inst_col, skinned, kind) = match &n.kind {
                Kind::Mesh { geo, mats } => (Some(geo.clone()), mats.clone(), false, false, false, None),
                Kind::Instanced { geo, mats, inst } => (Some(geo.clone()), mats.clone(), true, inst.colors.is_some(), false, None),
                Kind::Skinned { geo, mats, skeleton, .. } => (Some(geo.clone()), mats.clone(), false, false, skeleton.is_some(), None),
                Kind::Points { geo, mat } => (Some(geo.clone()), vec![mat.clone()], false, false, false, Some(3)),
                Kind::Line { geo, mat, .. } => (Some(geo.clone()), vec![mat.clone()], false, false, false, Some(5)), // draw_item's kind_v for 50/51/52
                Kind::Sprite { mat, .. } => (None, vec![mat.clone()], false, false, false, Some(4)),
                _ => return,
            };
            let morphs = !n.morph_influences.is_empty();
            drop(n);
            for m in mats {
                keys.push(variant_for(&m, geo.as_ref(), inst, inst_col, skinned, morphs, kind, fog));
            }
        });
        for k in keys {
            self.pipeline(k);
        }
    }

    /// Render `scene` from `camera` into `target` (the window's drawable, or an
    /// offscreen texture for screenshots), then composite `overlays`.
    pub fn render(
        &mut self,
        scene: &Obj,
        camera: &Obj,
        globals: &RenderGlobals,
        overlays: &[OverlayOp],
        target: &ProtocolObject<dyn MTLTexture>,
        out_w: usize,
        out_h: usize,
    ) -> Retained<ProtocolObject<dyn MTLCommandBuffer>> {
        let t_start = std::time::Instant::now();
        self.frame_no += 1;
        if self.frame_no.is_multiple_of(120) {
            self.sweep();
        }
        // frame arena: wait for its last use
        self.arena_i = (self.arena_i + 1) % self.arenas.len();
        if let Some(cb) = self.arenas[self.arena_i].cb.take() {
            cb.waitUntilCompleted();
        }
        self.arenas[self.arena_i].off = 0;

        let pr = globals.pixel_ratio.clamp(0.25, 2.0);
        let rw = ((out_w as f64) * pr).round().max(1.0) as usize;
        let rh = ((out_h as f64) * pr).round().max(1.0) as usize;
        self.ensure_targets(rw, rh);

        // ---- matrices (scene and camera world matrices)
        scene.update_matrix_world(false);
        if camera.parent().is_none() {
            camera.update_matrix_world(false);
        }
        let cam_world = camera.matrix_world();
        let view = cam_world.inverted();
        let cam = camera.r().kind.clone();
        let proj_gl = match &cam {
            Kind::Camera(c) => c.projection(),
            _ => Mat4::IDENTITY,
        };
        // OpenGL clip z [-1,1] → Metal [0,1]
        let mut zfix = Mat4::IDENTITY;
        zfix.e[10] = 0.5;
        zfix.e[14] = 0.5;
        let proj = Mat4::mul(&zfix, &proj_gl);
        let proj_screen = Mat4::mul(&proj_gl, &view);
        let frustum = Frustum::from_matrix(&proj_screen);

        // ---- collect (see `collect`)
        let mut items: Vec<Item> = vec![];
        let mut ambient = [0.0f64; 3];
        let mut hemis: Vec<HemiU> = vec![];
        let mut pts: Vec<PointU> = vec![];
        let mut pt_sph: Vec<(Vec3, f64)> = vec![];
        let mut spots: Vec<SpotU> = vec![];
        let mut sp_sph: Vec<(Vec3, f64)> = vec![];
        let fog = globals.fog;
        collect(scene, &frustum, &proj_screen, &view, 0, &mut items, &mut |n: &Node| {
            let mw = n.matrix_world;
            let wp = Vec3::new(mw.e[12], mw.e[13], mw.e[14]);
            match &n.kind {
                Kind::AmbientLight(l) => {
                    ambient[0] += l.color.r * l.intensity;
                    ambient[1] += l.color.g * l.intensity;
                    ambient[2] += l.color.b * l.intensity;
                }
                Kind::HemisphereLight(l) => {
                    let mut d = wp;
                    d.transform_direction(&view);
                    hemis.push(HemiU {
                        sky: c4(&l.color, l.intensity),
                        ground: c4(&l.ground_color, l.intensity),
                        dir: [d.x as f32, d.y as f32, d.z as f32, 0.0],
                    });
                }
                Kind::PointLight(l) => {
                    let mut vp = wp;
                    vp.apply_matrix4(&view);
                    pts.push(PointU {
                        pos: [vp.x as f32, vp.y as f32, vp.z as f32, l.distance as f32],
                        color: [
                            (l.color.r * l.intensity) as f32,
                            (l.color.g * l.intensity) as f32,
                            (l.color.b * l.intensity) as f32,
                            l.decay as f32,
                        ],
                    });
                    pt_sph.push((
                        wp,
                        if l.distance > 0.0 && l.intensity != 0.0 {
                            l.distance
                        } else if l.intensity == 0.0 {
                            -1.0
                        } else {
                            f64::INFINITY
                        },
                    ));
                }
                Kind::SpotLight(l) => {
                    let mut vp = wp;
                    vp.apply_matrix4(&view);
                    let tw = l.target.as_ref().map(|t| t.r().matrix_world.get_position()).unwrap_or(Vec3::ZERO);
                    let mut dir = wp;
                    dir.sub(&tw);
                    dir.transform_direction(&view);
                    let cone = l.angle.cos();
                    let pen = (l.angle * (1.0 - l.penumbra)).cos();
                    spots.push(SpotU {
                        pos: [vp.x as f32, vp.y as f32, vp.z as f32, l.distance as f32],
                        dir: [dir.x as f32, dir.y as f32, dir.z as f32, l.decay as f32],
                        color: [(l.color.r * l.intensity) as f32, (l.color.g * l.intensity) as f32, (l.color.b * l.intensity) as f32, 0.0],
                        cone: [cone as f32, pen as f32, 0.0, 0.0],
                    });
                    sp_sph.push((
                        wp,
                        if l.distance > 0.0 && l.intensity != 0.0 {
                            l.distance
                        } else if l.intensity == 0.0 {
                            -1.0
                        } else {
                            f64::INFINITY
                        },
                    ));
                }
                _ => {}
            }
        });
        // stable sorts: opaque by group, render order, material, then near to far;
        // transparent by group, render order, then far to near
        let (mut opaque, mut transp): (Vec<Item>, Vec<Item>) = items.into_iter().partition(|i| !i.transparent);
        opaque.sort_by(|a, b| {
            a.group_order
                .cmp(&b.group_order)
                .then(a.render_order.cmp(&b.render_order))
                .then(a.mat_id.cmp(&b.mat_id))
                .then(a.z.partial_cmp(&b.z).unwrap_or(std::cmp::Ordering::Equal))
                .then(a.id.cmp(&b.id))
        });
        transp.sort_by(|a, b| {
            a.group_order
                .cmp(&b.group_order)
                .then(a.render_order.cmp(&b.render_order))
                .then(b.z.partial_cmp(&a.z).unwrap_or(std::cmp::Ordering::Equal))
                .then(a.id.cmp(&b.id))
        });

        // ---- frame uniforms
        let mut fu = FrameU::default();
        fu.view = mat_f32(&view);
        fu.proj = mat_f32(&proj);
        fu.ambient = [ambient[0] as f32, ambient[1] as f32, ambient[2] as f32, 0.0];
        if let Some((c, d)) = fog {
            fu.fog = [c.r as f32, c.g as f32, c.b as f32, d as f32];
        }
        let ll = &globals.ll;
        fu.ll_grid = [ll.grid[0] as f32, ll.grid[1] as f32, ll.grid[2] as f32, ll.grid[3] as f32];
        fu.ll_amb = [ll.amb_d.x as f32, ll.amb_d.y as f32, ll.amb_d.z as f32, 0.0];
        fu.ll_sky = [ll.sky_d.x as f32, ll.sky_d.y as f32, ll.sky_d.z as f32, 0.0];
        fu.ll_gnd = [ll.gnd_d.x as f32, ll.gnd_d.y as f32, ll.gnd_d.z as f32, 0.0];
        fu.ll_fog_lit = [ll.fog_lit.r as f32, ll.fog_lit.g as f32, ll.fog_lit.b as f32, 0.0];
        fu.ll_params = [if ll.on { 1.0 } else { 0.0 }, ll.scale as f32, ll.fog_ratio as f32, if globals.floor_gaps { 1.0 } else { 0.0 }];
        fu.viewport = [rw as f32, rh as f32, rh as f32 * 0.5, 0.0];
        fu.counts = [hemis.len() as u32, pts.len() as u32, spots.len() as u32, 0];
        fu.look = [globals.floor_occlusion as f32, globals.corner_occlusion as f32, globals.floor_base as f32, globals.floor_step as f32];
        if hemis.is_empty() {
            hemis.push(HemiU::default());
        }
        if pts.is_empty() {
            pts.push(PointU::default());
        }
        if spots.is_empty() {
            spots.push(SpotU::default());
        }
        self.info = RenderInfo { lights: (pt_sph.len() + sp_sph.len()) as u32, programs: self.pipes.len() as u32, ..Default::default() };

        let cb = self.queue.commandBuffer().unwrap();
        let (pbuf, poff) = self.arena_alloc(bytes_of_slice(&pts));
        let (sbuf, soff) = self.arena_alloc(bytes_of_slice(&spots));
        let (hbuf, hoff) = self.arena_alloc(bytes_of_slice(&hemis));
        let ll_tex = match &ll.map {
            Some(t) => {
                let (id, _) = self.gpu_tex(t);
                Some(self.texs[&id].tex.clone())
            }
            None => None,
        };
        let ll_samp = self.sampler(ll.map.as_ref().map(|t| t.r().clone()).as_ref());

        // ---- the 3D pass
        let rp = MTLRenderPassDescriptor::new();
        unsafe {
            let ca = rp.colorAttachments().objectAtIndexedSubscript(0);
            ca.setTexture(self.msaa_color.as_deref());
            ca.setResolveTexture(self.scene_tex.as_deref());
            ca.setLoadAction(MTLLoadAction::Clear);
            ca.setStoreAction(MTLStoreAction::MultisampleResolve);
            let bg = &globals.background;
            ca.setClearColor(MTLClearColor {
                red: linear_to_srgb(bg.r),
                green: linear_to_srgb(bg.g),
                blue: linear_to_srgb(bg.b),
                alpha: 1.0,
            });
            let da = rp.depthAttachment();
            da.setTexture(self.msaa_depth.as_deref());
            da.setLoadAction(MTLLoadAction::Clear);
            da.setStoreAction(MTLStoreAction::DontCare);
            da.setClearDepth(1.0);
        }
        let enc = cb.renderCommandEncoderWithDescriptor(&rp).unwrap();
        enc.setFrontFacingWinding(MTLWinding::CounterClockwise);
        unsafe {
            enc.setVertexBytes_length_atIndex(NonNull::new(&fu as *const FrameU as *mut c_void).unwrap(), std::mem::size_of::<FrameU>(), 0);
            enc.setFragmentBytes_length_atIndex(
                NonNull::new(&fu as *const FrameU as *mut c_void).unwrap(),
                std::mem::size_of::<FrameU>(),
                0,
            );
            enc.setFragmentBuffer_offset_atIndex(Some(&hbuf), hoff, 3);
            enc.setFragmentBuffer_offset_atIndex(Some(&pbuf), poff, 4);
            enc.setFragmentBuffer_offset_atIndex(Some(&sbuf), soff, 5);
            enc.setFragmentTexture_atIndex(Some(ll_tex.as_deref().unwrap_or(&self.white_tex)), 1);
            enc.setFragmentSamplerState_atIndex(Some(&ll_samp), 1);
        }
        let lights = (pt_sph, sp_sph);
        for it in opaque.iter().chain(transp.iter()) {
            self.draw_item(&enc, it, &view, fog.is_some(), &lights);
        }
        enc.endEncoding();

        // ---- composite to the target
        let rp2 = MTLRenderPassDescriptor::new();
        unsafe {
            let ca = rp2.colorAttachments().objectAtIndexedSubscript(0);
            ca.setTexture(Some(target));
            ca.setLoadAction(MTLLoadAction::DontCare);
            ca.setStoreAction(MTLStoreAction::Store);
        }
        let enc2 = cb.renderCommandEncoderWithDescriptor(&rp2).unwrap();
        let p0 = self.comp_pipeline(0);
        enc2.setRenderPipelineState(&p0);
        let lin =
            self.sampler(Some(&Texture { min_filter: Filter::Linear, generate_mipmaps: false, ..Texture::from_image(Image::default()) }));
        let near = self.sampler(Some(&Texture {
            min_filter: Filter::Nearest,
            mag_filter: Filter::Nearest,
            generate_mipmaps: false,
            ..Texture::from_image(Image::default())
        }));
        let rep = self.sampler(Some(&Texture {
            min_filter: Filter::Linear,
            generate_mipmaps: false,
            wrap_s: Wrap::Repeat,
            wrap_t: Wrap::Repeat,
            ..Texture::from_image(Image::default())
        }));
        let full = [0.0f32, 0.0, 1.0, 1.0];
        // the canvas's CSS filter chain: count + four 3×4 matrices
        let mut sf = [0f32; 4 + 48];
        let nf = globals.scene_filter.len().min(4);
        sf[0] = nf as f32;
        for (k, m) in globals.scene_filter.iter().take(4).enumerate() {
            sf[4 + k * 12..4 + k * 12 + 12].copy_from_slice(m);
        }
        unsafe {
            enc2.setVertexBytes_length_atIndex(NonNull::new(full.as_ptr() as *mut c_void).unwrap(), 16, 0);
            enc2.setFragmentBytes_length_atIndex(NonNull::new(sf.as_ptr() as *mut c_void).unwrap(), std::mem::size_of_val(&sf), 0);
            enc2.setFragmentTexture_atIndex(self.scene_tex.as_deref(), 0);
            enc2.setFragmentSamplerState_atIndex(Some(&lin), 0);
            enc2.drawPrimitives_vertexStart_vertexCount(MTLPrimitiveType::TriangleStrip, 0, 4);
        }
        for op in overlays {
            #[repr(C)]
            struct P {
                a: [f32; 4],
                b: [f32; 4],
                c: [f32; 4],
                d: [f32; 4],
                e: [f32; 4],
                f: [f32; 4],
                g: [f32; 4],
                h: [f32; 4],
                i: [f32; 4],
                j: [f32; 4],
            }
            let mut p = P {
                a: [0.0; 4],
                b: [0.0; 4],
                c: [0.0; 4],
                d: [0.0; 4],
                e: [0.0; 4],
                f: [0.0; 4],
                g: [0.0; 4],
                h: [0.0; 4],
                i: [0.0; 4],
                j: [0.0; 4],
            };
            let (pipe, rect) = match op {
                OverlayOp::Image { image, rect, opacity, blend, tile, offset, smooth } => {
                    let tex = self.overlay_tex(image);
                    unsafe {
                        enc2.setFragmentTexture_atIndex(Some(&tex), 0);
                        let s = if *smooth { if tile[0] != 1.0 || tile[1] != 1.0 { &rep } else { &lin } } else { &near };
                        enc2.setFragmentSamplerState_atIndex(Some(s), 0);
                    }
                    p.a = [*opacity, *blend as u32 as f32, tile[0], tile[1]];
                    p.b = [offset[0], offset[1], 0.0, 0.0];
                    (1, *rect)
                }
                OverlayOp::Fill { color, blend } => {
                    p.a = *color;
                    p.b = [*blend as u32 as f32, 0.0, 0.0, 0.0];
                    (2, full)
                }
                OverlayOp::Radial { center, radius, stops, blend, opacity } => {
                    p.a = [center[0], center[1], radius[0], radius[1]];
                    p.b = [*blend as u32 as f32, stops.len().min(4) as f32, *opacity, 0.0];
                    let mut ts = [1.0f32; 4];
                    let mut cs = [[0.0f32; 4]; 4];
                    for (k, (t, c)) in stops.iter().take(4).enumerate() {
                        ts[k] = *t;
                        cs[k] = *c;
                    }
                    p.c = ts;
                    p.d = cs[0];
                    p.e = cs[1];
                    p.f = cs[2];
                    p.g = cs[3];
                    (3, full)
                }
                OverlayOp::Aberration { offset_px, opacity } => {
                    unsafe {
                        enc2.setFragmentTexture_atIndex(self.scene_tex.as_deref(), 0);
                        enc2.setFragmentSamplerState_atIndex(Some(&lin), 0);
                    }
                    p.a = [*offset_px / out_w.max(1) as f32, *opacity, 0.0, 0.0];
                    (4, full)
                }
            };
            let pl = self.comp_pipeline(pipe);
            enc2.setRenderPipelineState(&pl);
            unsafe {
                enc2.setVertexBytes_length_atIndex(NonNull::new(rect.as_ptr() as *mut c_void).unwrap(), 16, 0);
                enc2.setFragmentBytes_length_atIndex(NonNull::new(&p as *const P as *mut c_void).unwrap(), std::mem::size_of::<P>(), 0);
                enc2.drawPrimitives_vertexStart_vertexCount(MTLPrimitiveType::TriangleStrip, 0, 4);
            }
        }
        enc2.endEncoding();
        self.arenas[self.arena_i].cb = Some(cb.clone());
        self.info.frame_ms = t_start.elapsed().as_secs_f64() * 1000.0;
        cb
    }

    fn overlay_tex(&mut self, img: &OverlayImage) -> MTex {
        if let Some((v, t, w, h)) = self.overlay_texs.get(&img.id) {
            if *v == img.version && *w == img.width && *h == img.height {
                return t.clone();
            }
            if *w == img.width && *h == img.height {
                let t = t.clone();
                upload(&t, img.width as usize, img.height as usize, MTLPixelFormat::RGBA8Unorm, img.premul_rgba);
                self.overlay_texs.insert(img.id, (img.version, t.clone(), img.width, img.height));
                return t;
            }
        }
        let t = make_texture(
            &self.device,
            img.width.max(1) as usize,
            img.height.max(1) as usize,
            MTLPixelFormat::RGBA8Unorm,
            false,
            img.premul_rgba,
        );
        self.overlay_texs.insert(img.id, (img.version, t.clone(), img.width, img.height));
        t
    }

    fn draw_item(
        &mut self,
        enc: &ProtocolObject<dyn MTLRenderCommandEncoder>,
        it: &Item,
        view: &Mat4,
        scene_fog: bool,
        lights: &(Vec<(Vec3, f64)>, Vec<(Vec3, f64)>),
    ) {
        let n = it.obj.r();
        let (kind_i, inst, skinned) = match &n.kind {
            Kind::Mesh { .. } => (None, None, None),
            Kind::Instanced { inst, .. } => (None, Some(inst.clone()), None),
            // attached bind mode: the bind-matrix inverse is the mesh's inverse world matrix, refreshed every frame
            Kind::Skinned { skeleton, bind_matrix, .. } => {
                (None, None, skeleton.clone().map(|s| (s, *bind_matrix, n.matrix_world.inverted())))
            }
            Kind::Points { .. } => (Some(3), None, None),
            Kind::Line { segments, looped, .. } => (
                Some(if *segments {
                    51
                } else if *looped {
                    52
                } else {
                    50
                }),
                None,
                None,
            ),
            Kind::Sprite { center, .. } => {
                let _ = center;
                (Some(4), None, None)
            }
            _ => return,
        };
        let morph_inf = n.morph_influences.clone();
        let use_wear = n.use_wear;
        let sprite_center = if let Kind::Sprite { center, .. } = &n.kind { *center } else { Vec2::new(0.5, 0.5) };
        drop(n);
        if let Some(inst) = &inst
            && inst.count == 0
        {
            return;
        }
        let m = it.mat.r().clone();
        let geo_id = it.geo.as_ref().map(|g| self.gpu_geo(g));
        let line_kind = kind_i.filter(|k| *k >= 50);
        let kind_v = kind_i.map(|k| if k >= 50 { 5 } else { k });
        let mut key = variant_for(
            &it.mat,
            it.geo.as_ref(),
            inst.is_some(),
            inst.as_ref().map(|i| i.colors.is_some()).unwrap_or(false),
            skinned.is_some(),
            !morph_inf.is_empty(),
            kind_v,
            scene_fog,
        );
        if let Some(gid) = geo_id {
            key.v.morphs = key.v.morphs.min(self.geos[&gid].morphs as i32);
            if key.v.morphs == 0 {
                key.v.morph_nrm = false;
            }
        }
        let pipe = self.pipeline(key);
        enc.setRenderPipelineState(&pipe);
        let ds = self.depth_state(m.depth_test, m.depth_write);
        enc.setDepthStencilState(Some(&ds));
        enc.setCullMode(match (key.v.kind, m.side) {
            (3, _) | (5, _) => MTLCullMode::None,
            (_, Side::Front) => MTLCullMode::Back,
            (_, Side::Back) => MTLCullMode::Front,
            (_, Side::Double) => MTLCullMode::None,
        });
        if m.polygon_offset {
            enc.setDepthBias_slopeScale_clamp((m.polygon_offset_units * 1.0) as f32 * 0.0000005, m.polygon_offset_factor as f32, 0.0);
        } else {
            enc.setDepthBias_slopeScale_clamp(0.0, 0.0, 0.0);
        }

        // object uniforms
        let mw = it.world;
        let mv = Mat4::mul(view, &mw);
        let mut nm3 = Mat3::default();
        nm3.get_normal_matrix(&mv);
        let e = nm3.e;
        let normal4 = [
            e[0] as f32,
            e[1] as f32,
            e[2] as f32,
            0.0,
            e[3] as f32,
            e[4] as f32,
            e[5] as f32,
            0.0,
            e[6] as f32,
            e[7] as f32,
            e[8] as f32,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
        ];
        let mut morph = [0.0f32; 8];
        for (i, v) in morph_inf.iter().take(8).enumerate() {
            morph[i] = *v as f32;
        }
        let (bind, bind_inv) = match &skinned {
            Some((_, b, bi)) => (mat_f32(b), mat_f32(bi)),
            None => (Mat4::IDENTITY.to_f32(), Mat4::IDENTITY.to_f32()),
        };
        let ou = ObjectU {
            model: mat_f32(&mw),
            model_view: mat_f32(&mv),
            normal: normal4,
            bind,
            bind_inv,
            morph,
            sprite: [sprite_center.x as f32, sprite_center.y as f32, m.rotation as f32, 0.0],
            use_wear: [
                use_wear.handled as f32,
                use_wear.chips as f32,
                use_wear.floor as f32,
                if inst.as_ref().is_some_and(|i| i.use_wear.is_some()) { 1.0 } else { 0.0 },
            ],
        };
        // material uniforms
        let mut mu = MaterialU::default();
        mu.diffuse = [m.color.r as f32, m.color.g as f32, m.color.b as f32, m.opacity as f32];
        let ei = m.emissive_intensity;
        mu.emissive = [(m.emissive.r * ei) as f32, (m.emissive.g * ei) as f32, (m.emissive.b * ei) as f32, 0.0];
        mu.specular = [m.specular.r as f32, m.specular.g as f32, m.specular.b as f32, m.shininess.max(1e-4) as f32];
        mu.surf = [m.wear as f32, if m.tone_mapped { 1.0 } else { 0.0 }, m.wear_use as f32, 0.0];
        mu.map_t0 = [1.0, 0.0, 0.0, 0.0];
        mu.map_t1 = [0.0, 1.0, 0.0, 0.0];
        let mut flip = false;
        let mut map_tex = None;
        let mut map_samp = None;
        if let Some(t) = &m.map {
            let (tid, fy) = self.gpu_tex(t);
            flip = fy;
            let tt = t.r();
            let u = tt.uv_transform();
            mu.map_t0 = [u[0] as f32, u[3] as f32, u[6] as f32, 0.0];
            mu.map_t1 = [u[1] as f32, u[4] as f32, u[7] as f32, 0.0];
            let s = {
                let tc = tt.clone();
                drop(tt);
                self.sampler(Some(&tc))
            };
            map_tex = Some(self.texs[&tid].tex.clone());
            map_samp = Some(s);
        }
        mu.misc = [m.alpha_test as f32, m.size as f32, m.bump_scale as f32, if flip { 1.0 } else { 0.0 }];
        let mut bump_tex = None;
        let mut bump_samp = None;
        match &m.patch {
            Patch::ArchUV { scale_x, scale_y, wallpaper, detail, plane_shift } => {
                mu.arch = [*scale_x as f32, *scale_y as f32, if *wallpaper { 1.0 } else { 0.0 }, *plane_shift as f32];
                if let Some(d) = detail {
                    let (tid, _) = self.gpu_tex(&d.map);
                    let s = {
                        let tc = d.map.r().clone();
                        self.sampler(Some(&tc))
                    };
                    bump_tex = Some(self.texs[&tid].tex.clone());
                    bump_samp = Some(s);
                    mu.patch[0] = (1.0 / d.metres.max(1e-3)) as f32;
                    mu.patch[1] = d.tint as f32;
                    mu.patch[2] = d.relief as f32;
                }
            }
            Patch::SkinDetail { map, scale, .. } | Patch::WatcherSheet { map, scale, .. } => {
                let bm = m.bump_map.clone().unwrap_or(map.clone());
                let (tid, _) = self.gpu_tex(&bm);
                let s = {
                    let tc = bm.r().clone();
                    self.sampler(Some(&tc))
                };
                bump_tex = Some(self.texs[&tid].tex.clone());
                bump_samp = Some(s);
                mu.patch[0] = *scale as f32;
            }
            _ => {}
        }
        if matches!(m.patch, Patch::Shadow { .. }) {
            let pu = m.patch_uniforms.borrow();
            mu.patch[0] = pu.f[0];
            mu.patch[1] = pu.f[1];
        }
        if matches!(m.patch, Patch::WindowView | Patch::Glass) {
            mu.patch = m.patch_uniforms.borrow().f;
        }
        if let Some(p) = &m.portal {
            mu.portal = p.borrow().f;
        }
        // light list: lights whose reach touches this object
        let mut ll = LightListU { count: 0, pad: [0; 3], idx: [0; 96] };
        if key.v.kind == 1 || key.v.kind == 2 {
            let (c, r) = (it.sphere.center, it.sphere.radius);
            for (i, (p, d)) in lights.0.iter().enumerate() {
                if *d < 0.0 {
                    continue;
                }
                if (d.is_infinite() || r < 0.0 || p.distance_to(&c) <= d + r) && (ll.count as usize) < 96 {
                    ll.idx[ll.count as usize] = i as u16;
                    ll.count += 1;
                }
            }
            for (i, (p, d)) in lights.1.iter().enumerate() {
                if *d < 0.0 {
                    continue;
                }
                if (d.is_infinite() || r < 0.0 || p.distance_to(&c) <= d + r) && (ll.count as usize) < 96 {
                    ll.idx[ll.count as usize] = 32768 + i as u16;
                    ll.count += 1;
                }
            }
        }
        unsafe {
            enc.setVertexBytes_length_atIndex(
                NonNull::new(&ou as *const ObjectU as *mut c_void).unwrap(),
                std::mem::size_of::<ObjectU>(),
                1,
            );
            enc.setVertexBytes_length_atIndex(
                NonNull::new(&mu as *const MaterialU as *mut c_void).unwrap(),
                std::mem::size_of::<MaterialU>(),
                2,
            );
            enc.setFragmentBytes_length_atIndex(
                NonNull::new(&mu as *const MaterialU as *mut c_void).unwrap(),
                std::mem::size_of::<MaterialU>(),
                2,
            );
            enc.setFragmentBytes_length_atIndex(
                NonNull::new(&ll as *const LightListU as *mut c_void).unwrap(),
                std::mem::size_of::<LightListU>(),
                6,
            );
            enc.setFragmentTexture_atIndex(Some(map_tex.as_deref().unwrap_or(&self.dummy_tex)), 0);
            let ds = self.sampler(None);
            enc.setFragmentSamplerState_atIndex(Some(map_samp.as_deref().unwrap_or(&ds)), 0);
            enc.setFragmentTexture_atIndex(Some(bump_tex.as_deref().unwrap_or(&self.dummy_tex)), 2);
            enc.setFragmentSamplerState_atIndex(Some(bump_samp.as_deref().unwrap_or(&ds)), 2);
        }
        // sprites: no geometry
        if key.v.kind == 4 {
            unsafe {
                for i in (3..14).chain([15]) {
                    enc.setVertexBuffer_offset_atIndex(Some(&self.dummy_buf), 0, i);
                }
                let vc: u32 = 4;
                enc.setVertexBytes_length_atIndex(NonNull::new(&vc as *const u32 as *mut c_void).unwrap(), 4, 14);
                enc.drawPrimitives_vertexStart_vertexCount(MTLPrimitiveType::TriangleStrip, 0, 4);
            }
            self.info.calls += 1;
            self.info.triangles += 2;
            return;
        }
        let gid = geo_id.unwrap();
        // bones
        let mut bone_buf = None;
        if let Some((sk, _, _)) = &skinned {
            let mut data: Vec<f32> = Vec::with_capacity(sk.bones.len() * 16);
            for (b, inv) in sk.bones.iter().zip(sk.bone_inverses.iter()) {
                let m = Mat4::mul(&b.matrix_world(), inv);
                data.extend_from_slice(&m.to_f32());
            }
            bone_buf = Some(self.arena_alloc(bytemuck_f32(&data)));
        }
        // instances
        let mut inst_bufs = None;
        if let Some(inst) = &inst {
            let oid = it.obj.id();
            let fresh = self.insts.get(&oid).map(|e| e.version != inst.version).unwrap_or(true);
            if fresh {
                let mut data: Vec<f32> = Vec::with_capacity(inst.matrices.len() * 16);
                for mm in &inst.matrices {
                    data.extend_from_slice(&mm.to_f32());
                }
                let cols = inst.colors.as_ref().map(|cs| {
                    let v: Vec<f32> = cs.iter().flat_map(|c| [c.r as f32, c.g as f32, c.b as f32]).collect();
                    new_buffer(&self.device, bytemuck_f32(&v))
                });
                let wear = inst.use_wear.as_ref().map(|ws| {
                    let v: Vec<f32> = ws.iter().flat_map(|w| [w.handled as f32, w.chips as f32, w.floor as f32, 0.0]).collect();
                    new_buffer(&self.device, bytemuck_f32(&v))
                });
                self.insts.insert(
                    oid,
                    GpuInst {
                        weak: Rc::downgrade(&it.obj.0),
                        version: inst.version,
                        mats: new_buffer(&self.device, bytemuck_f32(&data)),
                        cols,
                        wear,
                    },
                );
            }
            let e = &self.insts[&oid];
            inst_bufs = Some((e.mats.clone(), e.cols.clone(), e.wear.clone()));
        }
        let g = &self.geos[&gid];
        let db = &self.dummy_buf;
        let vcount = g.vcount as u32;
        unsafe {
            enc.setVertexBuffer_offset_atIndex(Some(g.position.as_deref().unwrap_or(db)), 0, 3);
            enc.setVertexBuffer_offset_atIndex(Some(g.normal.as_deref().unwrap_or(db)), 0, 4);
            enc.setVertexBuffer_offset_atIndex(Some(g.uv.as_deref().unwrap_or(db)), 0, 5);
            enc.setVertexBuffer_offset_atIndex(Some(g.color.as_deref().unwrap_or(db)), 0, 6);
            match &inst_bufs {
                Some((mb, cb, wb)) => {
                    enc.setVertexBuffer_offset_atIndex(Some(mb), 0, 7);
                    enc.setVertexBuffer_offset_atIndex(Some(cb.as_deref().unwrap_or(db)), 0, 8);
                    enc.setVertexBuffer_offset_atIndex(Some(wb.as_deref().unwrap_or(db)), 0, 15);
                }
                None => {
                    enc.setVertexBuffer_offset_atIndex(Some(db), 0, 7);
                    enc.setVertexBuffer_offset_atIndex(Some(db), 0, 8);
                    enc.setVertexBuffer_offset_atIndex(Some(db), 0, 15);
                }
            }
            enc.setVertexBuffer_offset_atIndex(Some(g.skin_idx.as_deref().unwrap_or(db)), 0, 9);
            enc.setVertexBuffer_offset_atIndex(Some(g.skin_w.as_deref().unwrap_or(db)), 0, 10);
            match &bone_buf {
                Some((b, off)) => enc.setVertexBuffer_offset_atIndex(Some(b), *off, 11),
                None => enc.setVertexBuffer_offset_atIndex(Some(db), 0, 11),
            }
            enc.setVertexBuffer_offset_atIndex(Some(g.morph_pos.as_deref().unwrap_or(db)), 0, 12);
            enc.setVertexBuffer_offset_atIndex(Some(g.morph_nrm.as_deref().unwrap_or(db)), 0, 13);
            enc.setVertexBytes_length_atIndex(NonNull::new(&vcount as *const u32 as *mut c_void).unwrap(), 4, 14);
        }
        let prim = match (key.v.kind, line_kind) {
            (3, _) => MTLPrimitiveType::Point,
            (5, Some(51)) => MTLPrimitiveType::Line,
            (5, _) => MTLPrimitiveType::LineStrip,
            _ => MTLPrimitiveType::Triangle,
        };
        let (mut start, mut count) = (0usize, g.icount);
        if let Some((s, c)) = it.group {
            start = s as usize;
            count = (c as usize).min(g.icount.saturating_sub(start));
        }
        {
            let gg = it.geo.as_ref().unwrap().r();
            let ds = gg.draw_start as usize;
            let dc = gg.draw_count as usize;
            let rs = start.max(ds);
            let re = (start + count).min(ds.saturating_add(dc));
            start = rs;
            count = re.saturating_sub(rs);
        }
        if count == 0 {
            return;
        }
        let instances = inst.as_ref().map(|i| i.count.min(i.matrices.len())).unwrap_or(1);
        unsafe {
            match &g.index {
                Some(ib) => enc.drawIndexedPrimitives_indexCount_indexType_indexBuffer_indexBufferOffset_instanceCount(
                    prim,
                    count,
                    MTLIndexType::UInt32,
                    ib,
                    start * 4,
                    instances,
                ),
                None => enc.drawPrimitives_vertexStart_vertexCount_instanceCount(prim, start, count, instances),
            }
        }
        self.info.calls += 1;
        match prim {
            MTLPrimitiveType::Point => self.info.points += (count * instances) as u64,
            MTLPrimitiveType::Line | MTLPrimitiveType::LineStrip => self.info.lines += (count * instances) as u64,
            _ => self.info.triangles += (count / 3 * instances) as u64,
        }
    }

    /// Read a texture back (screenshots). BGRA → RGBA.
    pub fn read_texture(&self, tex: &ProtocolObject<dyn MTLTexture>) -> (usize, usize, Vec<u8>) {
        let (w, h) = (tex.width(), tex.height());
        let mut out = vec![0u8; w * h * 4];
        unsafe {
            tex.getBytes_bytesPerRow_fromRegion_mipmapLevel(
                NonNull::new(out.as_mut_ptr() as *mut c_void).unwrap(),
                w * 4,
                MTLRegion { origin: MTLOrigin { x: 0, y: 0, z: 0 }, size: MTLSize { width: w, height: h, depth: 1 } },
                0,
            );
        }
        for px in out.as_chunks_mut::<4>().0 {
            px.swap(0, 2);
            px[3] = 255;
        }
        (w, h, out)
    }
    /// An offscreen target the size of a window (screenshots, headless runs).
    pub fn offscreen_target(&self, w: usize, h: usize) -> MTex {
        unsafe {
            let d =
                MTLTextureDescriptor::texture2DDescriptorWithPixelFormat_width_height_mipmapped(MTLPixelFormat::BGRA8Unorm, w, h, false);
            d.setUsage(MTLTextureUsage::RenderTarget | MTLTextureUsage::ShaderRead);
            d.setStorageMode(MTLStorageMode::Shared);
            self.device.newTextureWithDescriptor(&d).unwrap()
        }
    }
}

fn c4(c: &Color, i: f64) -> [f32; 4] {
    [(c.r * i) as f32, (c.g * i) as f32, (c.b * i) as f32, 0.0]
}

fn variant_for(
    mat: &Mat,
    geo: Option<&crate::geometry::Geo>,
    instanced: bool,
    inst_color: bool,
    skinned: bool,
    morphs: bool,
    kind_override: Option<i32>,
    scene_fog: bool,
) -> PipeKey {
    let m = mat.r();
    let kind = kind_override.unwrap_or(match m.kind {
        MatKind::Basic | MatKind::LineBasic | MatKind::Depth => 0,
        MatKind::Lambert | MatKind::Standard => 1,
        MatKind::Phong => 2,
        MatKind::Points => 3,
        MatKind::Sprite => 4,
    });
    let (has_n, has_uv, vc_size, morph_n, morph_rel, nmorph) = match geo {
        Some(g) => {
            let g = g.r();
            (
                !g.normal.is_empty(),
                !g.uv.is_empty(),
                if g.color.is_empty() {
                    0
                } else if g.color_size == 4 {
                    4
                } else {
                    3
                },
                !g.morph_normal.is_empty(),
                g.morph_relative,
                g.morph_position.len().min(8) as i32,
            )
        }
        None => (false, true, 0, false, false, 0),
    };
    let vcolor = if m.vertex_colors { if vc_size == 4 { 4 } else { 3 } } else { 0 };
    let patch = match &m.patch {
        Patch::None => 0,
        Patch::ArchUV { .. } => 1,
        Patch::SkinDetail { .. } => 2,
        Patch::SmilerEye => 3,
        Patch::WatcherSheet { .. } => 4,
        Patch::Shadow { edge } => {
            if *edge {
                6
            } else {
                5
            }
        }
        Patch::Stairwell { .. } => 7,
        Patch::WindowView => 8,
        Patch::Glass => 9,
        Patch::Named(_) => 0,
    };
    let blend = if m.blending == Blending::Normal && !m.transparent {
        0
    } else {
        match m.blending {
            Blending::None => 0,
            Blending::Normal => 1,
            Blending::Additive => 2,
            Blending::Subtractive => 3,
            Blending::Multiply => 4,
        }
    };
    let v = VariantKey {
        kind,
        map: m.map.is_some(),
        vcolor,
        instanced,
        inst_color,
        skinned,
        morphs: if morphs { nmorph } else { 0 },
        fog: m.fog && scene_fog,
        alpha_test: m.alpha_test > 0.0,
        double: m.side == Side::Double,
        flip: m.side == Side::Back,
        flat: m.flat_shading,
        ll: true,
        patch,
        normal: has_n,
        uv: has_uv,
        opaque: !m.transparent && m.blending == Blending::Normal,
        bump: m.bump_map.is_some()
            || matches!(m.patch, Patch::SkinDetail { .. } | Patch::WatcherSheet { .. } | Patch::ArchUV { detail: Some(_), .. }),
        morph_nrm: morph_n && morphs,
        morph_rel,
        size_atten: m.size_attenuation,
        vc_data: vc_size > 0,
        wear: m.wear > 0.0 && (kind == 1 || kind == 2),
        portal: m.portal.is_some(),
    };
    PipeKey { v, blend, color_write: m.color_write }
}

// Visible-only traversal: gathers lights, frustum-culls renderables and records their view depth for sorting.
fn collect(
    o: &Obj,
    frustum: &Frustum,
    proj_screen: &Mat4,
    view: &Mat4,
    group_order: i32,
    items: &mut Vec<Item>,
    light: &mut dyn FnMut(&Node),
) {
    // an instanced mesh's bounding sphere is computed on its first frustum test
    if o.is_instanced() && o.visible() {
        let missing = matches!(&o.r().kind, Kind::Instanced { inst, .. } if inst.bounding_sphere.is_none());
        if missing {
            o.compute_instanced_bounding_sphere();
        }
    }
    let n = o.r();
    if !n.visible {
        return;
    }
    let mut go = group_order;
    let mw = n.matrix_world;
    match &n.kind {
        Kind::Group => go = n.render_order,
        Kind::AmbientLight(_) | Kind::HemisphereLight(_) | Kind::PointLight(_) | Kind::SpotLight(_) | Kind::DirectionalLight(_) => {
            light(&n)
        }
        Kind::Sprite { mat, .. } => {
            let mut sph = Sphere { center: Vec3::ZERO, radius: std::f64::consts::FRAC_1_SQRT_2 };
            sph.apply_matrix4(&mw);
            if (!n.frustum_culled || frustum.intersects_sphere(&sph)) && mat.r().visible {
                let mut p = mw.get_position();
                p.apply_matrix4(proj_screen);
                let m = mat.clone();
                let mid = m.id();
                let tr = m.r().transparent;
                items.push(Item {
                    obj: o.clone(),
                    geo: None,
                    mat: m,
                    group: None,
                    group_order: go,
                    render_order: n.render_order,
                    z: p.z,
                    id: n.id,
                    mat_id: mid,
                    transparent: tr,
                    world: mw,
                    sphere: sph,
                });
            }
        }
        Kind::Mesh { geo, mats } | Kind::Skinned { geo, mats, .. } | Kind::Instanced { geo, mats, .. } => {
            let local = match &n.kind {
                Kind::Instanced { inst, .. } => inst.bounding_sphere.unwrap_or_else(|| instanced_sphere(geo, inst)),
                Kind::Skinned { .. } => {
                    let s = geo.bounding_sphere();
                    Sphere { center: s.center, radius: s.radius * 1.35 + 0.3 }
                }
                _ => geo.bounding_sphere(),
            };
            let mut sph = local;
            sph.apply_matrix4(&mw);
            if !n.frustum_culled || frustum.intersects_sphere(&sph) {
                let mut p = local.center;
                p.apply_matrix4(&mw);
                p.apply_matrix4(proj_screen);
                if mats.len() > 1 {
                    let groups = geo.r().groups.clone();
                    for gr in groups {
                        if let Some(m) = mats.get(gr.material_index as usize) {
                            if !m.r().visible {
                                continue;
                            }
                            let mid = m.id();
                            let tr = m.r().transparent;
                            items.push(Item {
                                obj: o.clone(),
                                geo: Some(geo.clone()),
                                mat: m.clone(),
                                group: Some((gr.start, gr.count)),
                                group_order: go,
                                render_order: n.render_order,
                                z: p.z,
                                id: n.id,
                                mat_id: mid,
                                transparent: tr,
                                world: mw,
                                sphere: sph,
                            });
                        }
                    }
                } else if let Some(m) = mats.first()
                    && m.r().visible
                {
                    let mid = m.id();
                    let tr = m.r().transparent;
                    items.push(Item {
                        obj: o.clone(),
                        geo: Some(geo.clone()),
                        mat: m.clone(),
                        group: None,
                        group_order: go,
                        render_order: n.render_order,
                        z: p.z,
                        id: n.id,
                        mat_id: mid,
                        transparent: tr,
                        world: mw,
                        sphere: sph,
                    });
                }
            }
        }
        Kind::Points { geo, mat } | Kind::Line { geo, mat, .. } => {
            let local = geo.bounding_sphere();
            let mut sph = local;
            sph.apply_matrix4(&mw);
            if (!n.frustum_culled || frustum.intersects_sphere(&sph)) && mat.r().visible {
                let mut p = local.center;
                p.apply_matrix4(&mw);
                p.apply_matrix4(proj_screen);
                let mid = mat.id();
                let tr = mat.r().transparent;
                items.push(Item {
                    obj: o.clone(),
                    geo: Some(geo.clone()),
                    mat: mat.clone(),
                    group: None,
                    group_order: go,
                    render_order: n.render_order,
                    z: p.z,
                    id: n.id,
                    mat_id: mid,
                    transparent: tr,
                    world: mw,
                    sphere: sph,
                });
            }
        }
        _ => {}
    }
    let kids = n.children.clone();
    drop(n);
    let _ = view;
    for k in &kids {
        collect(k, frustum, proj_screen, view, go, items, light);
    }
}

fn instanced_sphere(geo: &crate::geometry::Geo, inst: &InstanceData) -> Sphere {
    let gs = geo.bounding_sphere();
    let mut b = Box3::empty();
    for m in inst.matrices.iter().take(inst.count) {
        let mut s = gs;
        s.apply_matrix4(m);
        b.expand_by_point(&(s.center - Vec3::new(s.radius, s.radius, s.radius)));
        b.expand_by_point(&(s.center + Vec3::new(s.radius, s.radius, s.radius)));
    }
    if b.is_empty() {
        return Sphere { center: Vec3::ZERO, radius: 0.0 };
    }
    let c = b.get_center();
    let mut r: f64 = 0.0;
    for m in inst.matrices.iter().take(inst.count) {
        let mut s = gs;
        s.apply_matrix4(m);
        r = r.max(s.center.distance_to(&c) + s.radius);
    }
    Sphere { center: c, radius: r }
}

struct Frustum {
    planes: [(Vec3, f64); 6],
}
impl Frustum {
    fn from_matrix(m: &Mat4) -> Frustum {
        let e = &m.e;
        let mk = |x: f64, y: f64, z: f64, w: f64| {
            let l = (x * x + y * y + z * z).sqrt();
            (Vec3::new(x / l, y / l, z / l), w / l)
        };
        Frustum {
            planes: [
                mk(e[3] - e[0], e[7] - e[4], e[11] - e[8], e[15] - e[12]),
                mk(e[3] + e[0], e[7] + e[4], e[11] + e[8], e[15] + e[12]),
                mk(e[3] + e[1], e[7] + e[5], e[11] + e[9], e[15] + e[13]),
                mk(e[3] - e[1], e[7] - e[5], e[11] - e[9], e[15] - e[13]),
                mk(e[3] - e[2], e[7] - e[6], e[11] - e[10], e[15] - e[14]),
                mk(e[3] + e[2], e[7] + e[6], e[11] + e[10], e[15] + e[14]),
            ],
        }
    }
    fn intersects_sphere(&self, s: &Sphere) -> bool {
        for (n, d) in &self.planes {
            if n.dot(&s.center) + d < -s.radius {
                return false;
            }
        }
        true
    }
}

// ------------------------------------------------------------ Metal helpers
fn bytemuck_f32(v: &[f32]) -> &[u8] {
    unsafe { std::slice::from_raw_parts(v.as_ptr() as *const u8, v.len() * 4) }
}
fn bytemuck_u32(v: &[u32]) -> &[u8] {
    unsafe { std::slice::from_raw_parts(v.as_ptr() as *const u8, v.len() * 4) }
}
fn bytes_of_slice<T: Copy>(v: &[T]) -> &[u8] {
    unsafe { std::slice::from_raw_parts(v.as_ptr() as *const u8, std::mem::size_of_val(v)) }
}
fn new_buffer(d: &Dev, bytes: &[u8]) -> Buf {
    let len = bytes.len().max(16);
    let b = d.newBufferWithLength_options(len, MTLResourceOptions::StorageModeShared).expect("buffer");
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), b.contents().as_ptr() as *mut u8, bytes.len());
    }
    b
}
fn make_texture(d: &Dev, w: usize, h: usize, fmt: MTLPixelFormat, mips: bool, data: &[u8]) -> MTex {
    let t = unsafe {
        let desc = MTLTextureDescriptor::texture2DDescriptorWithPixelFormat_width_height_mipmapped(fmt, w, h, mips);
        desc.setUsage(MTLTextureUsage::ShaderRead);
        d.newTextureWithDescriptor(&desc).expect("texture")
    };
    upload(&t, w, h, fmt, data);
    t
}
fn upload(t: &ProtocolObject<dyn MTLTexture>, w: usize, h: usize, fmt: MTLPixelFormat, data: &[u8]) {
    let bpp = if fmt == MTLPixelFormat::R8Unorm { 1 } else { 4 };
    if data.len() < w * h * bpp {
        return;
    }
    unsafe {
        t.replaceRegion_mipmapLevel_withBytes_bytesPerRow(
            MTLRegion { origin: MTLOrigin { x: 0, y: 0, z: 0 }, size: MTLSize { width: w, height: h, depth: 1 } },
            0,
            NonNull::new(data.as_ptr() as *mut c_void).unwrap(),
            w * bpp,
        );
    }
}
