//! The lab's stage: a stretch of office floor and two walls, lit evenly so
//! motion and shape can be read (a study light, not the game's light map),
//! a camera that orbits a point, and offscreen frames to PNG.
use engine::dmath::DMath;
use engine::prelude::*;
use engine::render::{RenderGlobals, Renderer};
use objc2_metal::MTLCommandBuffer;

/// How the stage is lit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Light {
    /// even, bright: to read shape and motion
    Study,
    /// a dim room and one warm lamp from where you stand: the game's mood
    Lamp,
}

pub struct Stage {
    pub scene: Obj,
    pub camera: Obj,
    pub light: Light,
    lights: Obj,
    lamp: Obj,
}

/// A camera orbiting `target`: heading `yaw`, `pitch` up, `dist` away.
#[derive(Clone, Copy, Debug)]
pub struct Orbit {
    pub target: [f64; 3],
    pub yaw: f64,
    pub pitch: f64,
    pub dist: f64,
}

impl Orbit {
    pub fn eye(&self) -> [f64; 3] {
        let (sy, cy) = self.yaw.dsin_cos();
        let (sp, cp) = self.pitch.dsin_cos();
        [self.target[0] + sy * cp * self.dist, self.target[1] + sp * self.dist, self.target[2] + cy * cp * self.dist]
    }
}

fn carpet_tex() -> Tex {
    let mut c = Canvas::new(256, 256);
    c.set_fill_style("#8a7d5a");
    c.fill_rect(0, 0, 256, 256);
    // a tufted weave: fine, low-contrast, seeded by position only
    for y in 0..64 {
        for x in 0..64 {
            let h = ((x * 7919 + y * 104_729) % 97) as f64 / 97.0;
            c.set_fill_style(format!("rgba(40,32,18,{:.3})", 0.05 + h * 0.12));
            c.fill_rect(x as f64 * 4.0, y as f64 * 4.0, 2.0 + h * 2.0, 2.0);
        }
    }
    let t = c.to_tex();
    t.t().color_space = ColorSpace::Srgb;
    t.set_repeat_wrapping();
    t.t().repeat.set(30, 30);
    t
}

impl Stage {
    pub fn new(aspect: f64) -> Stage {
        let scene = new_scene();
        let floor = mesh(plane_geo!(60, 60), lambert!({ map: &carpet_tex() }));
        floor.rotation().x = -std::f64::consts::PI / 2.0;
        scene.add(&floor);
        let wall_mat = lambert!({ color: 0xb8ad86 });
        let back = mesh(plane_geo!(60, 3.2), wall_mat.clone());
        back.set_pos(0, 1.6, -6.0);
        scene.add(&back);
        let left = mesh(plane_geo!(60, 3.2), wall_mat);
        left.rotation().y = std::f64::consts::PI / 2.0;
        left.set_pos(-6.0, 1.6, 0);
        scene.add(&left);
        let camera = perspective_camera(50.0, aspect, 0.05, 60.0);
        let lights = group();
        lights.add(&ambient_light(0xb8b0a0, 0.6));
        lights.add(&hemisphere_light(0xeeeadf, 0x7a7058, 1.05));
        let key = directional_light(0xfff4e0, 1.5);
        key.set_pos(3.0, 6.0, 4.0);
        lights.add(&key);
        let fill = directional_light(0xc8d0e0, 0.35);
        fill.set_pos(-4.0, 3.0, -2.0);
        lights.add(&fill);
        scene.add(&lights);
        let lamp = group();
        lamp.add(&ambient_light(0x403a30, 0.18));
        let bulb = point_light!(0xffe2b0, 3.2, 14.0, 1.4);
        bulb.set_pos(1.2, 1.5, 4.0);
        lamp.add(&bulb);
        scene.add(&lamp);
        let mut s = Stage { scene, camera, light: Light::Study, lights, lamp };
        s.set_light(Light::Study);
        s
    }

    pub fn set_light(&mut self, l: Light) {
        self.light = l;
        self.lights.set_visible(l == Light::Study);
        self.lamp.set_visible(l == Light::Lamp);
    }

    pub fn look(&self, o: &Orbit) {
        let e = o.eye();
        self.camera.set_pos(e[0], e[1], e[2]);
        self.camera.look_at(o.target[0], o.target[1], o.target[2]);
    }

    pub fn globals(&self) -> RenderGlobals {
        let bg = if self.light == Light::Study { 0x2a2620 } else { 0x050403 };
        RenderGlobals {
            background: Color::hex(bg),
            fog: if self.light == Light::Lamp { Some((Color::hex(0x050403), 0.08)) } else { None },
            pixel_ratio: 1.0,
            ..Default::default()
        }
    }

    /// One frame offscreen, as RGBA bytes.
    pub fn render(&self, r: &mut Renderer, w: usize, h: usize) -> (usize, usize, Vec<u8>) {
        let target = r.offscreen_target(w, h);
        let cb = r.render(&self.scene, &self.camera, &self.globals(), &[], &target, w, h);
        cb.commit();
        cb.waitUntilCompleted();
        r.read_texture(&target)
    }
}

/// Frames laid in a grid, `cols` wide, on a dark ground: a contact sheet.
pub fn sheet(frames: &[(usize, usize, Vec<u8>)], cols: usize) -> (usize, usize, Vec<u8>) {
    let (fw, fh) = (frames[0].0, frames[0].1);
    let rows = frames.len().div_ceil(cols);
    let gap = 4;
    let (w, h) = (cols * fw + (cols + 1) * gap, rows * fh + (rows + 1) * gap);
    let mut px = vec![0u8; w * h * 4];
    for p in px.chunks_mut(4) {
        p.copy_from_slice(&[12, 11, 10, 255]);
    }
    for (i, (_, _, f)) in frames.iter().enumerate() {
        let (cx, cy) = (gap + (i % cols) * (fw + gap), gap + (i / cols) * (fh + gap));
        for y in 0..fh {
            let src = &f[y * fw * 4..(y + 1) * fw * 4];
            let at = ((cy + y) * w + cx) * 4;
            px[at..at + fw * 4].copy_from_slice(src);
        }
    }
    (w, h, px)
}

pub fn write_png(path: &str, img: &(usize, usize, Vec<u8>)) {
    if let Some(dir) = std::path::Path::new(path).parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    engine::png::write_rgba(path, img.0 as u32, img.1 as u32, &img.2).expect("write png");
}
