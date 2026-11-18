//! The Jahamun set on its own: walk it in a window, or render it headless.
//!
//! ```sh
//! cargo run --release                                   # walk (from the top of the stairs)
//! cargo run --release -- --spawn inside                 # top, plaza, portal, inside, deep, end
//! cargo run --release -- --shot out.png --at video      # one frame offscreen (a named view, or x,y,z,yaw,pitch)
//! cargo run --release -- --shots DIR                    # every named view, and a contact sheet
//! cargo run --release -- --stats                        # what the set is made of
//! cargo run --release -- --bench                        # each view's frame time, offscreen at 1920 x 1080
//! cargo run --release -- --frames 300                   # the window, closed after 300 frames (its frame rate)
//! ```
//!
//! Keys: W A S D or the arrows walk, Shift runs, the mouse looks (click to
//! capture it, Esc to let it go; Esc again quits), F fullscreen, 1-6 the
//! spawns, F2 a headless frame of the view (`target/jahamun-F2.png` at the workspace root).
//! The eye is 1.55 m over the feet (the video's phone, held to look).
use engine::app::{AppConfig, AppCtx, AppHandler, Input};
use engine::dmath::DMath;
use engine::prelude::*;
use engine::render::Renderer;
use jahamun::Set;
use lens::Lens;

mod lens;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_metal::{MTLCommandBuffer, MTLCreateSystemDefaultDevice};
use std::collections::BTreeSet;

const EYE: f64 = 1.55;
const WALK: f64 = 1.35;
const RUN: f64 = 3.2;

fn arg(args: &[String], name: &str) -> Option<String> {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned()
}

fn flag(args: &[String], name: &str) -> bool {
    args.iter().any(|a| a == name)
}

/// A view: eye position, heading (0 looks into the tunnel, -z; positive
/// turns left), pitch (up), vertical field of view, size.
#[derive(Clone, Copy, Debug)]
struct View {
    eye: [f64; 3],
    yaw: f64,
    pitch: f64,
    fov: f64,
    w: usize,
    h: usize,
}

/// The named views: the video's own (portrait, its phone's field), and the
/// set's places.
fn views() -> Vec<(&'static str, View)> {
    let p = |eye: [f64; 3], yaw: f64, pitch: f64| View { eye, yaw, pitch, fov: 62.0, w: 720, h: 1280 };
    let l = |eye: [f64; 3], yaw: f64, pitch: f64| View { eye, yaw, pitch, fov: 50.0, w: 1280, h: 720 };
    vec![
        ("video", p([-0.82, 0.2 + 1.5, -30.0], 0.03, -0.17)),
        ("video_back", p([-0.95, 0.2 + 1.5, -60.0], std::f64::consts::PI - 0.04, -0.15)),
        ("video_wall", p([-0.55, 0.2 + 1.45, -90.0], 0.55, -0.05)),
        ("top", l([-7.0, 7.2 + EYE, 23.5], -0.35, -0.22)),
        ("stairs", l([-4.75, 3.7 + EYE, 12.6], 0.0, -0.2)),
        ("plaza", l([-3.6, 0.2 + EYE, 18.0], -0.32, 0.02)),
        ("portal", l([-0.9, 0.2 + EYE, 4.0], 0.0, -0.04)),
        ("inside", l([-0.85, 0.2 + EYE, -40.0], 0.0, -0.06)),
        ("glass", l([-1.2, 0.2 + EYE, -120.0], -0.75, 0.05)),
        ("deep", l([-0.85, 0.2 + EYE, -300.0], 0.0, -0.06)),
        ("back", l([-0.85, 0.2 + EYE, -200.0], std::f64::consts::PI, -0.06)),
        ("end", l([-0.85, 0.2 + EYE, -476.0], 0.0, -0.05)),
        ("road", l([4.4, 1.6, 30.0], 0.0, 0.0)),
    ]
}

fn look(camera: &Obj, eye: [f64; 3], yaw: f64, pitch: f64) {
    let (sy, cy) = yaw.dsin_cos();
    let (sp, cp) = pitch.dsin_cos();
    camera.set_pos(eye[0], eye[1], eye[2]);
    camera.look_at(eye[0] - sy * cp, eye[1] + sp, eye[2] - cy * cp);
}

fn set_lens(camera: &Obj, fov: f64, aspect: f64) {
    let mut n = camera.n();
    if let engine::scene::Kind::Camera(c) = &mut n.kind {
        c.fov = fov;
        c.aspect = aspect;
    }
}

/// The camera's grade over the engine's own: a little more contrast, the
/// whites cool (the video's phone under the tunnel's LEDs).
fn graded(mut g: engine::render::RenderGlobals, supersample: f64) -> engine::render::RenderGlobals {
    let cool: [f32; 12] = [0.965, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.035, 0.0];
    g.scene_filter = vec![engine::render::css_filter::contrast(1.06), cool];
    g.pixel_ratio = supersample;
    g
}

fn renderer() -> Renderer {
    let device = MTLCreateSystemDefaultDevice().expect("a Metal device");
    Renderer::new(device)
}

/// One frame offscreen.
fn render(r: &mut Renderer, set: &mut Set, scene: &Obj, camera: &Obj, v: &View) -> (usize, usize, Vec<u8>) {
    set_lens(camera, v.fov, v.w as f64 / v.h as f64);
    look(camera, v.eye, v.yaw, v.pitch);
    set.update(Vec3::new(v.eye[0], v.eye[1], v.eye[2]));
    let target = r.offscreen_target(v.w, v.h);
    let lens = Lens::new();
    // twice: the first compiles what it needs
    for _ in 0..2 {
        let cb = r.render(scene, camera, &graded(set.globals(), 1.5), &lens.overlays(0, v.w, v.h), &target, v.w, v.h);
        cb.commit();
        cb.waitUntilCompleted();
    }
    r.read_texture(&target)
}

fn write_png(path: &str, img: &(usize, usize, Vec<u8>)) {
    if let Some(dir) = std::path::Path::new(path).parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    engine::png::write_rgba(path, img.0 as u32, img.1 as u32, &img.2).expect("write png");
    println!("wrote {path}");
}

/// Frames side by side at one height, on a dark ground.
fn sheet(frames: &[(usize, usize, Vec<u8>)], cols: usize, cell_h: usize) -> (usize, usize, Vec<u8>) {
    let gap = 6;
    let scaled: Vec<(usize, usize, Vec<u8>)> = frames
        .iter()
        .map(|(w, h, px)| {
            let nh = cell_h;
            let nw = (*w as f64 * nh as f64 / *h as f64).round() as usize;
            let mut out = vec![0u8; nw * nh * 4];
            for y in 0..nh {
                for x in 0..nw {
                    // box filter over the source pixels this one covers
                    let (x0, x1) = (x * w / nw, ((x + 1) * w / nw).max(x * w / nw + 1));
                    let (y0, y1) = (y * h / nh, ((y + 1) * h / nh).max(y * h / nh + 1));
                    let mut acc = [0u32; 4];
                    for sy in y0..y1 {
                        for sx in x0..x1 {
                            let i = (sy * w + sx) * 4;
                            for k in 0..4 {
                                acc[k] += px[i + k] as u32;
                            }
                        }
                    }
                    let n = ((x1 - x0) * (y1 - y0)) as u32;
                    let o = (y * nw + x) * 4;
                    for k in 0..4 {
                        out[o + k] = (acc[k] / n) as u8;
                    }
                }
            }
            (nw, nh, out)
        })
        .collect();
    let rows = scaled.len().div_ceil(cols);
    let row_w: Vec<usize> = (0..rows).map(|r| scaled.iter().skip(r * cols).take(cols).map(|f| f.0 + gap).sum::<usize>() + gap).collect();
    let w = *row_w.iter().max().unwrap_or(&1);
    let h = rows * (cell_h + gap) + gap;
    let mut px = vec![0u8; w * h * 4];
    for p in px.as_chunks_mut::<4>().0 {
        *p = [14, 13, 12, 255];
    }
    for r in 0..rows {
        let mut cx = gap;
        let cy = gap + r * (cell_h + gap);
        for f in scaled.iter().skip(r * cols).take(cols) {
            for y in 0..f.1 {
                let src = &f.2[y * f.0 * 4..(y + 1) * f.0 * 4];
                let at = ((cy + y) * w + cx) * 4;
                px[at..at + f.0 * 4].copy_from_slice(src);
            }
            cx += f.0 + gap;
        }
    }
    (w, h, px)
}

fn parse_view(s: &str) -> Option<View> {
    if let Some((_, v)) = views().into_iter().find(|(n, _)| *n == s) {
        return Some(v);
    }
    let n: Vec<f64> = s.split(',').filter_map(|x| x.trim().parse().ok()).collect();
    (n.len() >= 5).then(|| View { eye: [n[0], n[1], n[2]], yaw: n[3], pitch: n[4], fov: *n.get(5).unwrap_or(&55.0), w: 1280, h: 720 })
}

struct App {
    set: Set,
    scene: Obj,
    camera: Obj,
    feet: [f64; 3],
    eye_y: f64,
    yaw: f64,
    pitch: f64,
    keys: BTreeSet<String>,
    bob: f64,
    locked: bool,
    shot: bool,
    lens: Lens,
    frame: u64,
    /// quit after this many frames (a smoke test of the window)
    quit_after: Option<u64>,
    started: std::time::Instant,
}

impl App {
    fn go(&mut self, spawn: &str) {
        if let Some((p, yaw)) = self.set.spawn(spawn) {
            self.feet = p;
            self.eye_y = p[1] + EYE;
            self.yaw = yaw;
            self.pitch = -0.05;
        }
    }
}

impl AppHandler for App {
    fn init(&mut self, _ctx: &mut AppCtx) {}

    fn frame(&mut self, ctx: &mut AppCtx, dt: f64) -> Option<Retained<ProtocolObject<dyn MTLCommandBuffer>>> {
        let dt = dt.clamp(0.0, 0.1);
        let held = |k: &[&str]| k.iter().any(|k| self.keys.contains(*k));
        let mut fwd: f64 = 0.0;
        let mut side: f64 = 0.0;
        if held(&["KeyW", "ArrowUp"]) {
            fwd += 1.0;
        }
        if held(&["KeyS", "ArrowDown"]) {
            fwd -= 1.0;
        }
        if held(&["KeyD", "ArrowRight"]) {
            side += 1.0;
        }
        if held(&["KeyA", "ArrowLeft"]) {
            side -= 1.0;
        }
        let speed = if held(&["ShiftLeft", "ShiftRight"]) { RUN } else { WALK };
        let len = (fwd * fwd + side * side).sqrt();
        let (sy, cy) = self.yaw.dsin_cos();
        let mut moved = 0.0;
        if len > 0.0 {
            let (f, s) = (fwd / len * speed * dt, side / len * speed * dt);
            let dx = -sy * f + cy * s;
            let dz = -cy * f - sy * s;
            let before = self.feet;
            self.feet = self.set.walk.step(self.feet, dx, dz);
            moved = ((self.feet[0] - before[0]).dpowi(2) + (self.feet[2] - before[2]).dpowi(2)).sqrt();
        }
        // the eye follows the feet up and down the stairs a little behind
        let target = self.feet[1] + EYE;
        self.eye_y += (target - self.eye_y) * (1.0 - (-dt * 10.0).dexp());
        self.bob += moved * 1.75 * std::f64::consts::PI;
        let amp = if moved > 0.0 { 0.025 } else { 0.0 };
        let eye = [self.feet[0], self.eye_y + amp * self.bob.dsin().abs() - amp * 0.5, self.feet[2]];
        let (w, h) = ctx.physical_size();
        set_lens(&self.camera, 62.0, w as f64 / h.max(1) as f64);
        look(&self.camera, eye, self.yaw, self.pitch);
        self.set.update(Vec3::new(eye[0], eye[1], eye[2]));
        if self.shot {
            self.shot = false;
            let v = View { eye, yaw: self.yaw, pitch: self.pitch, fov: 62.0, w: 1280, h: 720 };
            let mut r = renderer();
            let img = render(&mut r, &mut self.set, &self.scene, &self.camera, &v);
            write_png(concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/jahamun-F2.png"), &img);
            set_lens(&self.camera, 62.0, w as f64 / h.max(1) as f64);
        }
        let d = ctx.drawable?;
        self.frame += 1;
        if self.quit_after.is_some_and(|n| self.frame >= n) {
            let s = self.started.elapsed().as_secs_f64();
            println!("{} frames in {s:.2} s ({:.0} fps) at {w} x {h}", self.frame, self.frame as f64 / s);
            ctx.quit();
        }
        let cb = ctx.renderer.render(
            &self.scene,
            &self.camera,
            &graded(self.set.globals(), 1.0),
            &self.lens.overlays(self.frame, w, h),
            d,
            w,
            h,
        );
        Some(cb)
    }

    fn input(&mut self, ctx: &mut AppCtx, ev: Input) {
        match ev {
            Input::KeyDown(k, repeat) => {
                if !repeat {
                    match k.as_str() {
                        "Escape" => {
                            if self.locked {
                                ctx.set_pointer_lock(false);
                            } else {
                                ctx.quit();
                            }
                        }
                        "KeyF" => {
                            let fs = !ctx.fullscreen;
                            ctx.set_fullscreen(fs);
                        }
                        "F2" => self.shot = true,
                        "Digit1" => self.go("top"),
                        "Digit2" => self.go("plaza"),
                        "Digit3" => self.go("portal"),
                        "Digit4" => self.go("inside"),
                        "Digit5" => self.go("deep"),
                        "Digit6" => self.go("end"),
                        _ => {}
                    }
                }
                self.keys.insert(k);
            }
            Input::KeyUp(k) => {
                self.keys.remove(&k);
            }
            Input::MouseDown(_) => {
                if !self.locked {
                    ctx.set_pointer_lock(true);
                }
            }
            Input::MouseMove(dx, dy) => {
                if self.locked {
                    self.yaw -= dx * 0.0022;
                    self.pitch = (self.pitch - dy * 0.0022).clamp(-1.4, 1.4);
                }
            }
            Input::PointerLock(on) => self.locked = on,
            Input::Blur => self.keys.clear(),
            Input::CloseRequested => ctx.quit(),
            _ => {}
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if flag(&args, "--help") || flag(&args, "-h") {
        println!(
            "{}",
            include_str!("main.rs")
                .lines()
                .take_while(|l| l.starts_with("//!"))
                .map(|l| l.trim_start_matches("//!").trim_start())
                .collect::<Vec<_>>()
                .join("\n")
        );
        return;
    }
    // canvases paint at once (the game's executable does the same)
    engine::texture::set_paint_on_demand(false);
    let t0 = std::time::Instant::now();
    let mut set = match Set::build() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("the set: {e}");
            std::process::exit(1);
        }
    };
    let scene = new_scene();
    scene.add(&set.root);
    let camera = perspective_camera(62.0, 16.0 / 9.0, 0.08, 1200.0);
    eprintln!("built the set in {:.2} s", t0.elapsed().as_secs_f64());

    if flag(&args, "--stats") {
        let mut meshes = 0;
        let mut tris = 0usize;
        set.root.traverse(&mut |o: &Obj| {
            if o.is_mesh()
                && let Some(g) = o.try_geo()
            {
                meshes += 1;
                tris += g.r().index.as_ref().map_or(0, |i| i.len() / 3);
            }
        });
        println!("{meshes} meshes, {tris} triangles placed; tunnel {} m; spawns {:?}", set.tunnel, set.spawns.keys().collect::<Vec<_>>());
        return;
    }
    if let Some(path) = arg(&args, "--shot") {
        let mut v = arg(&args, "--at").and_then(|s| parse_view(&s)).unwrap_or(views()[0].1);
        if let Some(sz) = arg(&args, "--size") {
            let p: Vec<usize> = sz.split('x').filter_map(|x| x.parse().ok()).collect();
            if p.len() == 2 {
                (v.w, v.h) = (p[0], p[1]);
            }
        }
        if let Some(f) = arg(&args, "--fov").and_then(|f| f.parse().ok()) {
            v.fov = f;
        }
        let mut r = renderer();
        let img = render(&mut r, &mut set, &scene, &camera, &v);
        write_png(&path, &img);
        return;
    }
    if flag(&args, "--bench") {
        // each named view rendered 60 times offscreen at 1920 x 1080, waited
        // for: the GPU's whole frame, as the window would draw it
        // (BENCH_HIDE=highlight,glass,sprites,...: what costs what)
        if let Ok(hide) = std::env::var("BENCH_HIDE") {
            let hide: Vec<String> = hide.split(',').map(String::from).collect();
            set.root.traverse(&mut |o: &Obj| {
                let name = if o.is_sprite() { "sprites".to_string() } else { o.try_mat().map(|m| m.r().name.clone()).unwrap_or_default() };
                if hide.contains(&name) {
                    o.set_visible(false);
                }
            });
        }
        let mut r = renderer();
        let lens = Lens::new();
        for (name, mut v) in views() {
            (v.w, v.h) = (1920, 1080);
            let _ = render(&mut r, &mut set, &scene, &camera, &v);
            let target = r.offscreen_target(v.w, v.h);
            let t = std::time::Instant::now();
            let (mut cpu, mut gpu) = (0.0, 0.0);
            for _ in 0..60 {
                let cb = r.render(&scene, &camera, &graded(set.globals(), 1.0), &lens.overlays(0, v.w, v.h), &target, v.w, v.h);
                cpu += r.info.frame_ms;
                cb.commit();
                cb.waitUntilCompleted();
                gpu += (cb.GPUEndTime() - cb.GPUStartTime()) * 1000.0;
            }
            let ms = t.elapsed().as_secs_f64() * 1000.0 / 60.0;
            println!(
                "{name:<12} {ms:6.2} ms a frame: encoding {:5.2} ms, GPU {:5.2} ms  ({} draw calls, {} triangles)",
                cpu / 60.0,
                gpu / 60.0,
                r.info.calls,
                r.info.triangles
            );
        }
        return;
    }
    if let Some(dir) = arg(&args, "--shots") {
        let mut r = renderer();
        let mut portrait = vec![];
        let mut land = vec![];
        let only: Option<Vec<String>> = arg(&args, "--only").map(|s| s.split(',').map(String::from).collect());
        for (name, v) in views() {
            if only.as_ref().is_some_and(|o| !o.iter().any(|x| x == name)) {
                continue;
            }
            let img = render(&mut r, &mut set, &scene, &camera, &v);
            write_png(&format!("{dir}/{name}.png"), &img);
            if v.h > v.w {
                portrait.push(img)
            } else {
                land.push(img)
            };
        }
        if !portrait.is_empty() {
            write_png(&format!("{dir}/sheet_video.png"), &sheet(&portrait, 3, 960));
        }
        if !land.is_empty() {
            write_png(&format!("{dir}/sheet.png"), &sheet(&land, 3, 405));
        }
        return;
    }

    let mut app = App {
        set,
        scene,
        camera,
        feet: [0.0; 3],
        eye_y: 0.0,
        yaw: 0.0,
        pitch: 0.0,
        keys: BTreeSet::new(),
        bob: 0.0,
        locked: false,
        shot: false,
        lens: Lens::new(),
        frame: 0,
        quit_after: arg(&args, "--frames").and_then(|n| n.parse().ok()),
        started: std::time::Instant::now(),
    };
    app.go(arg(&args, "--spawn").as_deref().unwrap_or("top"));
    engine::app::run(
        AppConfig { title: "Jahamun Tunnel — a set".into(), width: 1280.0, height: 720.0, fullscreen: flag(&args, "--fullscreen") },
        app,
    );
}
