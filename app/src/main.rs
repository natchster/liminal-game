//! The Backrooms — native macOS build.
//!
//! Usage: `backrooms [--fullscreen] [--width W --height H] [--hash '#nosound,scp=loop@2']
//!                   [--screenshot out.png [--frames N] [--level L] [--seed S] [--play]]`
use engine::app::{run, AppConfig, AppCtx, AppHandler, Input};
use engine::render::{OBlend, OverlayImage, OverlayOp, RenderGlobals, LocalLight, Renderer};
use game::dom::{HudLayer, LayerBlend};
use game::Game;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_metal::{MTLCommandBuffer, MTLCreateSystemDefaultDevice, MTLTexture};
use std::time::Instant;

/// Every prop placer, by its JS name.
fn register_placers(r: &mut Vec<(&'static str, game::state::PlacerFn)>) {
    placers_p00::register(r); placers_p01::register(r); placers_p02::register(r); placers_p03::register(r);
    placers_p04::register(r); placers_p05::register(r); placers_p06::register(r); placers_p07::register(r);
    placers_p08::register(r); placers_p09::register(r); placers_p10::register(r); placers_p11::register(r);
    placers_p12::register(r); placers_p13::register(r); placers_p14::register(r); placers_p15::register(r);
    placers_p16::register(r); placers_p17::register(r); placers_p18::register(r); placers_p19::register(r);
    placers_p20::register(r); placers_p21::register(r); placers_p22::register(r); placers_p23::register(r);
    placers_p24::register(r); placers_p25::register(r); placers_p26::register(r); placers_p27::register(r);
    placers_p28::register(r); placers_p29::register(r); placers_p30::register(r); placers_p31::register(r);
    placers_p32::register(r);
}

struct Options { dpr: f64, fullscreen: bool, width: f64, height: f64, hash: String, screenshot: Option<String>, frames: u32, level: Option<i32>, seed: Option<u32>, play: bool, dump_level: Option<String>, levels: Vec<i32> }
fn parse_args() -> Options {
    let mut o = Options { dpr: 1.0, fullscreen: false, width: 1280.0, height: 800.0, hash: String::new(), screenshot: None, frames: 90, level: None, seed: None, play: false, dump_level: None, levels: vec![] };
    let a: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < a.len() {
        let next = |i: &mut usize| { *i += 1; a.get(*i).cloned().unwrap_or_default() };
        match a[i].as_str() {
            "--fullscreen" | "-f" => o.fullscreen = true,
            "--windowed" => o.fullscreen = false,
            "--width" => o.width = next(&mut i).parse().unwrap_or(1280.0),
            "--height" => o.height = next(&mut i).parse().unwrap_or(800.0),
            "--hash" => o.hash = next(&mut i),
            "--screenshot" => o.screenshot = Some(next(&mut i)),
            "--frames" => o.frames = next(&mut i).parse().unwrap_or(90),
            "--level" => o.level = next(&mut i).parse().ok(),
            "--seed" => o.seed = next(&mut i).parse().ok(),
            "--play" => o.play = true,
            "--dpr" => o.dpr = next(&mut i).parse().unwrap_or(1.0),
            "--dump-level" => o.dump_level = Some(next(&mut i)),
            "--levels" => o.levels = next(&mut i).split(',').filter_map(|x| x.parse().ok()).collect(),
            "--help" | "-h" => {
                println!("backrooms [--fullscreen] [--width W --height H] [--hash '#flags'] [--screenshot out.png [--frames N] [--level L] [--seed S] [--play]]");
                println!("          [--dump-level DIR --seed S --levels 1,2,3 [--screenshot x]]   (compare with tools/levelcheck/js-capture.mjs)");
                println!("          [--dpr 2]   headless modes: render at this device pixel ratio");
                std::process::exit(0);
            }
            _ => {}
        }
        i += 1;
    }
    if !o.hash.is_empty() && !o.hash.starts_with('#') { o.hash = format!("#{}", o.hash); }
    o
}

fn save_path() -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    std::path::Path::new(&home).join("Library/Application Support/Backrooms/localStorage.txt")
}

fn make_game(o: &Options) -> Game {
    let mut g = Game::new_bare();
    g.location_hash = o.hash.clone();
    g.storage_load(save_path());
    register_placers(&mut g.placers);
    g
}

fn globals(g: &mut Game) -> RenderGlobals {
    let scene_filter = g.scene_filter_matrices();
    RenderGlobals {
        scene_filter,
        background: g.background,
        fog: Some((g.fog_color, g.fog_density)),
        ll: LocalLight {
            on: g.ll_on > 0.5, map: g.ll_map.clone(), grid: g.ll_grid, scale: g.ll_scale,
            amb_d: g.ll_amb_d, sky_d: g.ll_sky_d, gnd_d: g.ll_gnd_d, fog_lit: g.ll_fog_lit, fog_ratio: g.ll_fog_ratio,
        },
        pixel_ratio: g.quality.pr / g.device_pixel_ratio.max(1.0),
    }
}

fn blend(b: LayerBlend) -> OBlend {
    match b {
        LayerBlend::Normal => OBlend::Normal, LayerBlend::Multiply => OBlend::Multiply, LayerBlend::Screen => OBlend::Screen,
        LayerBlend::Add => OBlend::Add, LayerBlend::Overlay => OBlend::Overlay, LayerBlend::SoftLight => OBlend::SoftLight,
        LayerBlend::ColorDodge => OBlend::ColorDodge, LayerBlend::Darken => OBlend::Darken, LayerBlend::Lighten => OBlend::Lighten,
    }
}

/// Render the game's scene and its page layers into `target`.
fn draw(g: &mut Game, r: &mut Renderer, target: &ProtocolObject<dyn objc2_metal::MTLTexture>, w_px: usize, h_px: usize) -> Retained<ProtocolObject<dyn MTLCommandBuffer>> {
    if g.want_warm_up { g.want_warm_up = false; let gl = globals(g); r.warm_up(&g.scene, &gl); }
    let gl = globals(g);
    let layers = g.hud_layers(g.inner_width, g.inner_height, g.device_pixel_ratio);
    // borrow the canvases for the duration of the render
    let guards: Vec<Option<std::cell::Ref<'_, engine::canvas::Canvas>>> = layers.iter().map(|l| match l { HudLayer::Canvas { canvas, .. } => Some(canvas.borrow()), _ => None }).collect();
    let mut ops: Vec<OverlayOp> = vec![];
    for (l, gd) in layers.iter().zip(guards.iter()) {
        match l {
            HudLayer::Canvas { id, version, rect, opacity, blend: b, tile, offset, smooth, .. } => {
                let c = gd.as_ref().unwrap();
                ops.push(OverlayOp::Image { image: OverlayImage { id: *id, version: *version, width: c.width() as u32, height: c.height() as u32, premul_rgba: c.premultiplied() }, rect: *rect, opacity: *opacity, blend: blend(*b), tile: *tile, offset: *offset, smooth: *smooth });
            }
            HudLayer::Fill { color, blend: b } => ops.push(OverlayOp::Fill { color: *color, blend: blend(*b) }),
            HudLayer::Radial { center, radius, stops, blend: b, opacity } => ops.push(OverlayOp::Radial { center: *center, radius: *radius, stops: stops.clone(), blend: blend(*b), opacity: *opacity }),
            HudLayer::Aberration { offset_px, opacity } => ops.push(OverlayOp::Aberration { offset_px: *offset_px, opacity: *opacity }),
        }
    }
    r.render(&g.scene, &g.camera, &gl, &ops, target, w_px, h_px)
}

struct App { g: Game, started: Instant, cursor: (f64, f64), last_hit: String }

impl AppHandler for App {
    fn init(&mut self, ctx: &mut AppCtx) {
        self.g.inner_width = ctx.width; self.g.inner_height = ctx.height; self.g.device_pixel_ratio = ctx.dpr;
        self.g.quality.max_pr = ctx.dpr.min(2.0); self.g.quality.pr = self.g.quality.max_pr;
        self.g.now_ms = 0.0;
        self.g.boot();
    }
    fn frame(&mut self, ctx: &mut AppCtx, dt: f64) -> Option<Retained<ProtocolObject<dyn MTLCommandBuffer>>> {
        let g = &mut self.g;
        g.now_ms = self.started.elapsed().as_secs_f64() * 1000.0;
        if (g.inner_width - ctx.width).abs() > 0.5 || (g.inner_height - ctx.height).abs() > 0.5 || (g.device_pixel_ratio - ctx.dpr).abs() > 0.01 {
            g.inner_width = ctx.width; g.inner_height = ctx.height; g.device_pixel_ratio = ctx.dpr;
            g.quality.max_pr = ctx.dpr.min(2.0); g.quality.pr = g.quality.pr.min(g.quality.max_pr);
            g.resize();
        }
        g.frame(dt);
        if let Some(l) = g.want_pointer_lock.take() { ctx.set_pointer_lock(l); }
        if let Some(f) = g.want_fullscreen.take() { ctx.set_fullscreen(f); }
        if g.want_quit { ctx.quit(); }
        let target = ctx.drawable?;
        let (w, h) = (target.width(), target.height());
        Some(draw(g, ctx.renderer, target, w, h))
    }
    fn input(&mut self, ctx: &mut AppCtx, ev: Input) {
        let g = &mut self.g;
        match ev {
            Input::KeyDown(code, repeat) => {
                // Cmd+Q / Cmd+F: the native window's own shortcuts
                if code == "KeyQ" && (g.key("MetaLeft") || g.key("MetaRight")) { ctx.quit(); return; }
                if (code == "KeyF" && (g.key("MetaLeft") || g.key("MetaRight")) && g.key("ControlLeft")) || code == "F11" { let fs = !ctx.fullscreen; ctx.set_fullscreen(fs); return; }
                g.key_down(&code, repeat);
            }
            Input::KeyUp(code) => g.key_up(&code),
            Input::MouseMove(dx, dy) => g.mouse_move(dx, dy),
            Input::CursorPos(x, y) => { self.cursor = (x, y); }
            Input::MouseDown(0) => {
                let hit = g.hud_hit_test(self.cursor.0, self.cursor.1);
                self.last_hit = hit.clone();
                g.click(&hit);
            }
            Input::PointerLock(on) => g.pointer_lock_changed(on),
            Input::Blur => g.blur(),
            Input::Resize => { g.inner_width = ctx.width; g.inner_height = ctx.height; g.device_pixel_ratio = ctx.dpr; g.resize(); }
            _ => {}
        }
        if let Some(l) = g.want_pointer_lock.take() { ctx.set_pointer_lock(l); }
        if let Some(f) = g.want_fullscreen.take() { ctx.set_fullscreen(f); }
    }
}

/// Headless: build a level and save a frame (no window).
fn screenshot(o: &Options, path: &str) {
    let device = MTLCreateSystemDefaultDevice().expect("Metal device");
    let mut r = Renderer::new(device);
    let mut g = make_game(o);
    let (w, h) = ((o.width * o.dpr) as usize, (o.height * o.dpr) as usize);
    g.inner_width = o.width; g.inner_height = o.height; g.device_pixel_ratio = o.dpr;
    g.quality.max_pr = o.dpr.min(2.0); g.quality.pr = g.quality.max_pr;
    g.boot();
    if let Some(s) = o.seed { g.run_seed = s; }
    if let Some(l) = o.level { g.init_level(l); } else if o.seed.is_some() { g.init_level(1); }
    if o.play { g.state = "playing".into(); g.hide_overlay(); }
    let target = r.offscreen_target(w, h);
    let t0 = Instant::now();
    for i in 0..o.frames {
        g.now_ms = (i as f64) * 1000.0 / 60.0;
        g.frame(1.0 / 60.0);
        let cb = draw(&mut g, &mut r, &target, w, h);
        cb.commit();
        if i + 1 == o.frames { cb.waitUntilCompleted(); }
    }
    let (tw, th, px) = r.read_texture(&target);
    engine::png::write_rgba(path, tw as u32, th as u32, &px).expect("write png");
    println!("wrote {path} ({tw}x{th}) after {} frames in {:.2}s — draw calls {}, triangles {}, lights {}, programs {}",
        o.frames, t0.elapsed().as_secs_f64(), r.info.calls, r.info.triangles, r.info.lights, r.info.programs);
}

/// `--dump-level DIR`: what tools/levelcheck/js-capture.mjs does in the
/// browser — start a run from `#seed=S`, then per level: trace the draws,
/// load it, write `rs_<seed>_L<n>.json`; with `--screenshot`, rebuild it, take
/// one fixed 1/60 s step, freeze, and save `rs_<seed>_L<n>.png`.
fn level_check(o: &Options, dir: &str) {
    let seed = o.seed.unwrap_or(4242);
    let mut g = make_game(o);
    g.location_hash = if o.hash.is_empty() { format!("#seed={seed}") } else { format!("#seed={seed}&{}", o.hash.trim_start_matches('#')) };
    let (w, h) = ((o.width * o.dpr) as usize, (o.height * o.dpr) as usize);
    g.inner_width = o.width; g.inner_height = o.height; g.device_pixel_ratio = o.dpr;
    g.quality.max_pr = o.dpr.min(2.0); g.quality.pr = g.quality.max_pr;
    let device = MTLCreateSystemDefaultDevice().expect("Metal device");
    let mut r = Renderer::new(device);
    g.boot();
    g.start_new_game();   // __game.begin() from the menu
    // the browser page has been running for a while when it is captured: let the
    // film grain (repainted every 55 ms, opacity eased) settle the same way
    for _ in 0..60 { g.update_grain(1.0 / 60.0); }
    std::fs::create_dir_all(dir).expect("create dump dir");
    let levels = if o.levels.is_empty() { vec![o.level.unwrap_or(1)] } else { o.levels.clone() };
    let shots = o.screenshot.is_some();
    let target = if shots { Some(r.offscreen_target(w, h)) } else { None };
    for &l in &levels {
        g.rand_trace(true);
        g.load_level(l);
        let dump = g.level_dump();
        g.rand_trace(false);
        let base = format!("{dir}/rs_{seed}_L{l}");
        std::fs::write(format!("{base}.json"), dump.to_string()).expect("write dump");
        let marks = dump["marks"].as_array().map(|a| a.len()).unwrap_or(0);
        let draws = dump["marks"].as_array().and_then(|a| a.last()).and_then(|m| m[1].as_u64()).unwrap_or(0);
        let objs = dump["objects"].as_array().map(|a| a.len()).unwrap_or(0);
        print!("level {l}: {marks} marks, {draws} draws, {objs} objects, scp={}, weather={}", dump["scp"], dump["weather"]);
        if let Some(t) = &target {
            g.load_level(l);
            g.fixed_step_dt = 1.0 / 60.0; g.frame(1.0 / 60.0); g.fixed_step_dt = 0.0;
            g.sim_frozen = true;
            let cb = draw(&mut g, &mut r, t, w, h);
            cb.commit(); cb.waitUntilCompleted();
            let (tw, th, px) = r.read_texture(t);
            engine::png::write_rgba(&format!("{base}.png"), tw as u32, th as u32, &px).expect("write png");
            g.sim_frozen = false;
            print!(" (+png)");
        }
        println!();
    }
}

fn main() {
    let o = parse_args();
    if let Some(d) = o.dump_level.clone() { level_check(&o, &d); return; }
    if let Some(p) = o.screenshot.clone() { screenshot(&o, &p); return; }
    let g = make_game(&o);
    run(AppConfig { title: "The Backrooms".into(), width: o.width, height: o.height, fullscreen: o.fullscreen },
        App { g, started: Instant::now(), cursor: (0.0, 0.0), last_hit: String::new() });
}
