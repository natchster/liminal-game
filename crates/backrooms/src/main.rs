//! The Backrooms — native macOS build.
//!
//! Usage: `backrooms [--fullscreen] [--width W --height H] [--flags 'nosound&scp=loop@2']
//!                   [--screenshot out.png [--frames N] [--level L] [--seed S] [--play]]`
use engine::app::{run, AppConfig, AppCtx, AppHandler, Input};
use engine::render::{LocalLight, OBlend, OverlayImage, OverlayOp, RenderGlobals, Renderer};
use game::ui::model::{HudLayer, LayerBlend};
use game::Game;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_metal::{MTLCommandBuffer, MTLCreateSystemDefaultDevice, MTLTexture};
use std::time::Instant;

/// Every prop placer, by its registered name.
fn register_placers(r: &mut Vec<(&'static str, game::base::state::PlacerFn)>) {
    placers_p00::register(r);
    placers_p01::register(r);
    placers_p02::register(r);
    placers_p03::register(r);
    placers_p04::register(r);
    placers_p05::register(r);
    placers_p06::register(r);
    placers_p07::register(r);
    placers_p08::register(r);
    placers_p09::register(r);
    placers_p10::register(r);
    placers_p11::register(r);
    placers_p12::register(r);
    placers_p13::register(r);
    placers_p14::register(r);
    placers_p15::register(r);
    placers_p16::register(r);
    placers_p17::register(r);
    placers_p18::register(r);
    placers_p19::register(r);
    placers_p20::register(r);
    placers_p21::register(r);
    placers_p22::register(r);
    placers_p23::register(r);
    placers_p24::register(r);
    placers_p25::register(r);
    placers_p26::register(r);
    placers_p27::register(r);
    placers_p28::register(r);
    placers_p29::register(r);
    placers_p30::register(r);
    placers_p31::register(r);
    placers_p32::register(r);
}

struct Options {
    scale: f64,
    fullscreen: bool,
    width: f64,
    height: f64,
    flags: String,
    screenshot: Option<String>,
    frames: u32,
    level: Option<i32>,
    seed: Option<u32>,
    play: bool,
    dump_level: Option<String>,
    levels: Vec<i32>,
    /// `--showcase smiler[:dist]`: pin a creature in front of the camera
    showcase: Option<(String, f64)>,
    /// `--showcase kind:dist:walk`: placed there once, then it goes about
    showcase_walk: bool,
    /// `--death`: with `--showcase`, it kills you 15 frames before the shot
    death: bool,
    /// `--screen pause|leave|settings|brightness|controls|title|dead|ending|notebook|notebook-typed`:
    /// with `--play`, that list or card goes up 45 frames before the shot (`Game::debug_screen`)
    screen: Option<String>,
    /// `--views`: with `--dump-level --screenshot`, one more PNG from inside each zone
    views: bool,
    /// `--views-at "x,z,yaw;…"`: with `--dump-level --screenshot`, a PNG from
    /// each of these poses (world metres, radians), `_v00` on: the same poses
    /// in two builds (`--views` prints the poses along the walls that lean)
    views_at: Vec<(f64, f64, f64)>,
    /// `--vending-demo hang|bin`: with `--views-at`, every lit vending
    /// machine is first played to that state (`Game::vending_demo`)
    vending_demo: Option<String>,
    /// `--record FILE`: play normally and record the session (inputs, frame
    /// times, telemetry, the director's events) to FILE; opt-in, local only
    record: Option<String>,
    /// `--replay FILE`: play a recorded session back; ArrowUp / ArrowDown mark
    /// tension rising / falling (saved to FILE.trace.json), Escape stops
    replay: Option<String>,
    /// `--bench FILE`: with `--seed` and `--levels`, time each level's build,
    /// its first frame and `bench_frames` frames along a fixed camera path
    bench: Option<String>,
    bench_frames: usize,
    /// `--bench-creatures FILE`: with `--seed`, time the creature pack (every
    /// kind that roams the rooms, two of each, all hunting you) on level 4
    bench_creatures: Option<String>,
    /// `--metrics FILE`: with `--seed` and `--levels`, build the levels as a
    /// run would and write their expressive-range measures (no rendering)
    metrics: Option<String>,
    /// `--fresh`: ignore the saved settings and progress, and seed the
    /// per-frame randomness: the same shot every time
    fresh: bool,
}
fn parse_args() -> Options {
    let mut o = Options {
        scale: 1.0,
        fullscreen: false,
        width: 1280.0,
        height: 800.0,
        flags: String::new(),
        screenshot: None,
        frames: 90,
        level: None,
        seed: None,
        play: false,
        dump_level: None,
        levels: vec![],
        showcase: None,
        showcase_walk: false,
        screen: None,
        death: false,
        views: false,
        views_at: vec![],
        vending_demo: None,
        fresh: false,
        metrics: None,
        bench: None,
        bench_creatures: None,
        bench_frames: 240,
        record: None,
        replay: None,
    };
    let a: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < a.len() {
        let next = |i: &mut usize| {
            *i += 1;
            a.get(*i).cloned().unwrap_or_default()
        };
        match a[i].as_str() {
            "--fullscreen" | "-f" => o.fullscreen = true,
            "--windowed" => o.fullscreen = false,
            "--width" => o.width = next(&mut i).parse().unwrap_or(1280.0),
            "--height" => o.height = next(&mut i).parse().unwrap_or(800.0),
            "--flags" => o.flags = next(&mut i),
            "--views-at" => {
                o.views_at = next(&mut i)
                    .split(';')
                    .filter_map(|p| match p.split(',').map(|v| v.trim().parse::<f64>()).collect::<Result<Vec<_>, _>>() {
                        Ok(v) if v.len() == 3 => Some((v[0], v[1], v[2])),
                        _ => None,
                    })
                    .collect()
            }
            "--vending-demo" => o.vending_demo = Some(next(&mut i)),
            "--screenshot" => o.screenshot = Some(next(&mut i)),
            "--frames" => o.frames = next(&mut i).parse().unwrap_or(90),
            "--level" => o.level = next(&mut i).parse().ok(),
            "--seed" => o.seed = next(&mut i).parse().ok(),
            "--play" => o.play = true,
            "--death" => o.death = true,
            "--screen" => o.screen = Some(next(&mut i)),
            "--showcase" => {
                let v = next(&mut i);
                let mut it = v.splitn(3, ':');
                let kind = it.next().unwrap_or("smiler").to_string();
                o.showcase = Some((kind, it.next().and_then(|d| d.parse().ok()).unwrap_or(3.5)));
                // `kind:dist:walk`: placed once, then left to go about (its walk)
                o.showcase_walk = it.next() == Some("walk");
            }
            "--scale" => o.scale = next(&mut i).parse().unwrap_or(1.0),
            "--dump-level" => o.dump_level = Some(next(&mut i)),
            "--levels" => o.levels = next(&mut i).split(',').filter_map(|x| x.parse().ok()).collect(),
            "--views" => o.views = true,
            "--fresh" => o.fresh = true,
            "--metrics" => o.metrics = Some(next(&mut i)),
            "--bench" => o.bench = Some(next(&mut i)),
            "--bench-creatures" => o.bench_creatures = Some(next(&mut i)),
            "--record" => o.record = Some(next(&mut i)),
            "--replay" => o.replay = Some(next(&mut i)),
            "--bench-frames" => o.bench_frames = next(&mut i).parse().unwrap_or(240),
            "--help" | "-h" => {
                println!("backrooms [--fullscreen] [--width W --height H] [--flags 'seed=4242&nosound&scp=loop@2'] [--screenshot out.png [--frames N] [--level L] [--seed S] [--play] [--showcase smiler[:dist[:walk]] [--death]]]");
                println!("          [--dump-level DIR --seed S --levels 1,2,3 [--screenshot x [--views]]]   writes a JSON description of each level (and a PNG with --screenshot, one per zone with --views)");
                println!("          [--scale 2]   headless modes: render at this backing scale (2 = Retina)");
                println!(
                    "          [--record FILE]   play and record the session (inputs, telemetry, events) to FILE, on this machine only"
                );
                println!("          [--replay FILE]   play a recorded session back; Up / Down arrows mark tension rising / falling (FILE.trace.json)");
                println!("          [--bench FILE --seed S --levels 1,2,3 [--bench-frames N]]   times build, first frame, CPU update and GPU per frame along a fixed path (JSON)");
                println!("          [--metrics FILE --seed S --levels 1,2,3]   writes each level's expressive-range measures as JSON (no rendering)");
                println!("          [--fresh]     headless modes: ignore the saved settings and progress, fixed per-frame randomness (reproducible shots)");
                std::process::exit(0);
            }
            _ => {}
        }
        i += 1;
    }
    o
}

fn save_path() -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    std::path::Path::new(&home).join("Library/Application Support/Backrooms/storage.txt")
}

fn make_game(o: &Options) -> Game {
    let mut g = Game::new_bare();
    g.launch_flags = o.flags.clone();
    let headless =
        o.screenshot.is_some() || o.dump_level.is_some() || o.metrics.is_some() || o.bench.is_some() || o.bench_creatures.is_some();
    // a recorded or replayed session starts like a fresh install, so the two
    // start alike (their per-frame seed is set in `main`)
    let session = o.record.is_some() || o.replay.is_some();
    if headless && o.fresh {
        // per-frame randomness from a fixed seed: the same shot every time
        game::base::rng::seed_native(0x5EED_5EED);
    }
    if !(session || (headless && o.fresh)) {
        g.storage_load(save_path());
    }
    if headless || session {
        // a headless run reads the save at most; a death in a showcase must
        // not end up in the player's progress
        g.storage_path = None;
    }
    if headless {
        // ... and never the player's settings: its frames must not depend on
        // how someone set the field of view or the brightness
        g.storage.remove(game::session::settings::SETTINGS_KEY);
    }
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
            on: g.ll_on > 0.5,
            map: g.ll_map.clone(),
            grid: g.ll_grid,
            scale: g.ll_scale,
            amb_d: g.ll_amb_d,
            sky_d: g.ll_sky_d,
            gnd_d: g.ll_gnd_d,
            fog_lit: g.ll_fog_lit,
            fog_ratio: g.ll_fog_ratio,
        },
        pixel_ratio: g.quality.pr / g.scale_factor.max(1.0),
        floor_occlusion: g.floor_occlusion(),
        corner_occlusion: g.corner_occlusion(),
    }
}

fn blend(b: LayerBlend) -> OBlend {
    match b {
        LayerBlend::Normal => OBlend::Normal,
        LayerBlend::Multiply => OBlend::Multiply,
        LayerBlend::Screen => OBlend::Screen,
        LayerBlend::Add => OBlend::Add,
        LayerBlend::Overlay => OBlend::Overlay,
        LayerBlend::SoftLight => OBlend::SoftLight,
        LayerBlend::ColorDodge => OBlend::ColorDodge,
        LayerBlend::Darken => OBlend::Darken,
        LayerBlend::Lighten => OBlend::Lighten,
    }
}

/// Render the game's scene and its HUD layers into `target`.
fn draw(
    g: &mut Game,
    r: &mut Renderer,
    target: &ProtocolObject<dyn objc2_metal::MTLTexture>,
    w_px: usize,
    h_px: usize,
) -> Retained<ProtocolObject<dyn MTLCommandBuffer>> {
    if g.want_warm_up {
        g.want_warm_up = false;
        let gl = globals(g);
        r.warm_up(&g.scene, &gl);
    }
    let gl = globals(g);
    let layers = g.hud_layers(g.view_width, g.view_height, g.scale_factor);
    // borrow the canvases for the duration of the render
    let guards: Vec<Option<std::cell::Ref<'_, engine::canvas::Canvas>>> = layers
        .iter()
        .map(|l| match l {
            HudLayer::Canvas { canvas, .. } => Some(canvas.borrow()),
            _ => None,
        })
        .collect();
    let mut ops: Vec<OverlayOp> = vec![];
    for (l, gd) in layers.iter().zip(guards.iter()) {
        match l {
            HudLayer::Canvas { id, version, rect, opacity, blend: b, tile, offset, smooth, .. } => {
                let c = gd.as_ref().unwrap();
                ops.push(OverlayOp::Image {
                    image: OverlayImage {
                        id: *id,
                        version: *version,
                        width: c.width() as u32,
                        height: c.height() as u32,
                        premul_rgba: c.premultiplied(),
                    },
                    rect: *rect,
                    opacity: *opacity,
                    blend: blend(*b),
                    tile: *tile,
                    offset: *offset,
                    smooth: *smooth,
                });
            }
            HudLayer::Fill { color, blend: b } => ops.push(OverlayOp::Fill { color: *color, blend: blend(*b) }),
            HudLayer::Radial { center, radius, stops, blend: b, opacity } => {
                ops.push(OverlayOp::Radial { center: *center, radius: *radius, stops: stops.clone(), blend: blend(*b), opacity: *opacity })
            }
            HudLayer::Aberration { offset_px, opacity } => ops.push(OverlayOp::Aberration { offset_px: *offset_px, opacity: *opacity }),
        }
    }
    r.render(&g.scene, &g.camera, &gl, &ops, target, w_px, h_px)
}

struct App {
    g: Game,
    started: Instant,
    cursor: (f64, f64),
    last_hit: String,
    /// `--record`: the session so far, and where it is saved
    recorder: Option<(game::debug::replay::Recorder, String)>,
    /// `--replay`: the session, the next frame, the trace file, its marks
    replaying: Option<Replaying>,
    frames: u64,
}

struct Replaying {
    rec: game::debug::replay::Recording,
    next: usize,
    trace: String,
    marks: Vec<serde_json::Value>,
}

impl App {
    fn save(&self) {
        if let Some((r, path)) = &self.recorder {
            let _ = std::fs::write(path, r.rec.to_json().to_string());
        }
        if let Some(rp) = &self.replaying {
            let _ =
                std::fs::write(&rp.trace, serde_json::json!({ "recording_frames": rp.rec.frames.len(), "marks": rp.marks }).to_string());
        }
    }
}

impl Drop for App {
    fn drop(&mut self) {
        self.save();
    }
}

impl AppHandler for App {
    fn init(&mut self, ctx: &mut AppCtx) {
        self.g.view_width = ctx.width;
        self.g.view_height = ctx.height;
        self.g.scale_factor = ctx.scale_factor;
        self.g.quality.max_pr = ctx.scale_factor.min(2.0);
        self.g.quality.pr = self.g.quality.max_pr;
        self.g.now_ms = 0.0;
        self.g.boot();
        // the title has the lobby's sound (a window needs no click to start audio)
        self.g.audio_init();
    }
    fn frame(&mut self, ctx: &mut AppCtx, dt: f64) -> Option<Retained<ProtocolObject<dyn MTLCommandBuffer>>> {
        self.frames += 1;
        if self.frames % 300 == 0 || self.g.want_quit {
            self.save();
        }
        if let Some(rp) = self.replaying.as_mut() {
            // the recorded step, clock and input; the window's are ignored
            let Some((rdt, now, input)) = rp.rec.frames.get(rp.next).cloned() else {
                println!("replay finished: {} tension marks in {}", rp.marks.len(), rp.trace);
                self.save();
                ctx.quit();
                return None;
            };
            rp.next += 1;
            game::debug::replay::replay_frame(&mut self.g, rdt, now, &input);
            let target = ctx.drawable?;
            let (w, h) = (target.width(), target.height());
            return Some(draw(&mut self.g, ctx.renderer, target, w, h));
        }
        let g = &mut self.g;
        g.now_ms = self.started.elapsed().as_secs_f64() * 1000.0;
        if (g.view_width - ctx.width).abs() > 0.5
            || (g.view_height - ctx.height).abs() > 0.5
            || (g.scale_factor - ctx.scale_factor).abs() > 0.01
        {
            g.view_width = ctx.width;
            g.view_height = ctx.height;
            g.scale_factor = ctx.scale_factor;
            g.quality.max_pr = ctx.scale_factor.min(2.0);
            g.quality.pr = g.quality.pr.min(g.quality.max_pr);
            g.resize();
        }
        g.frame(dt);
        if let Some((r, _)) = self.recorder.as_mut() {
            r.end_frame(g, dt, g.now_ms);
        }
        if let Some(l) = g.want_pointer_lock.take() {
            ctx.set_pointer_lock(l);
        }
        if let Some(f) = g.want_fullscreen.take() {
            ctx.set_fullscreen(f);
        }
        if g.want_quit {
            ctx.quit();
        }
        let target = ctx.drawable?;
        let (w, h) = (target.width(), target.height());
        Some(draw(g, ctx.renderer, target, w, h))
    }
    fn input(&mut self, ctx: &mut AppCtx, ev: Input) {
        use game::debug::replay::InputEvent;
        if let Some(rp) = self.replaying.as_mut() {
            // a replay only listens for the viewer's tension marks
            if let Input::KeyDown(code, false) = &ev {
                let dir = match code.as_str() {
                    "ArrowUp" => 1,
                    "ArrowDown" => -1,
                    "Escape" => {
                        self.save();
                        ctx.quit();
                        return;
                    }
                    _ => 0,
                };
                if dir != 0 {
                    rp.marks.push(serde_json::json!({ "t": self.g.perf_t, "frame": rp.next, "level": self.g.level_num, "tension": dir }));
                }
            }
            return;
        }
        // what reaches the game is recorded (the window's own shortcuts are not)
        let shortcut = match &ev {
            Input::KeyDown(code, _) => {
                let meta = self.g.key("MetaLeft") || self.g.key("MetaRight");
                (code == "KeyQ" && meta) || (code == "KeyF" && meta && self.g.key("ControlLeft")) || code == "F11"
            }
            _ => false,
        };
        // (a key's auto-repeat only keeps it held, so it is not recorded; a
        // mouse button is a key while the pointer is locked, a click otherwise)
        let locked = self.g.pointer_locked;
        if let Some((r, _)) = self.recorder.as_mut().filter(|_| !shortcut) {
            match &ev {
                // (a repeat of a key no longer held, after Command forgot it, is a press)
                Input::KeyDown(code, rep) if !*rep || !self.g.key(code) => r.push(InputEvent::Down(code.clone())),
                Input::KeyUp(code) => r.push(InputEvent::Up(code.clone())),
                Input::MouseMove(dx, dy) => r.push(InputEvent::Look(*dx, *dy)),
                Input::MouseDown(b) if locked => r.push(InputEvent::Down(format!("Mouse{b}"))),
                Input::MouseUp(b) if locked || self.g.key(&format!("Mouse{b}")) => r.push(InputEvent::Up(format!("Mouse{b}"))),
                Input::MouseDown(0) => r.push(InputEvent::Click(self.g.hud_hit_test(self.cursor.0, self.cursor.1))),
                Input::Wheel(d) => r.push(InputEvent::Wheel(*d)),
                Input::PointerLock(on) => r.push(InputEvent::Lock(*on)),
                Input::Blur => r.push(InputEvent::Blur),
                _ => {}
            }
        }
        let g = &mut self.g;
        match ev {
            Input::KeyDown(code, repeat) => {
                // Cmd+Q / Cmd+F: the native window's own shortcuts
                if code == "KeyQ" && (g.key("MetaLeft") || g.key("MetaRight")) {
                    ctx.quit();
                    return;
                }
                if (code == "KeyF" && (g.key("MetaLeft") || g.key("MetaRight")) && g.key("ControlLeft")) || code == "F11" {
                    let fs = !ctx.fullscreen;
                    ctx.set_fullscreen(fs);
                    return;
                }
                g.key_down(&code, repeat);
            }
            Input::KeyUp(code) => g.key_up(&code),
            Input::MouseMove(dx, dy) => g.mouse_move(dx, dy),
            Input::CursorPos(x, y) => {
                self.cursor = (x, y);
                // over a menu line: it is the chosen one (recorded, so a replay chooses alike)
                if g.menu_hover(x, y) {
                    if let (Some((r, _)), Some(i)) = (self.recorder.as_mut(), g.menu.items.get(g.menu.sel)) {
                        r.push(InputEvent::Hover(i.id.to_string()));
                    }
                }
            }
            // while the pointer is locked the buttons are keys (`Mouse0`..`Mouse2`)
            Input::MouseDown(b) if g.pointer_locked => g.key_down(&format!("Mouse{b}"), false),
            Input::MouseUp(b) if g.key(&format!("Mouse{b}")) => g.key_up(&format!("Mouse{b}")),
            Input::MouseDown(0) => {
                let hit = g.hud_hit_test(self.cursor.0, self.cursor.1);
                self.last_hit = hit.clone();
                g.click(&hit);
            }
            Input::Wheel(d) => g.wheel(d),
            Input::PointerLock(on) => g.pointer_lock_changed(on),
            Input::Blur => g.blur(),
            Input::Resize => {
                g.view_width = ctx.width;
                g.view_height = ctx.height;
                g.scale_factor = ctx.scale_factor;
                g.resize();
            }
            _ => {}
        }
        if let Some(l) = g.want_pointer_lock.take() {
            ctx.set_pointer_lock(l);
        }
        if let Some(f) = g.want_fullscreen.take() {
            ctx.set_fullscreen(f);
        }
    }
}

/// Headless: build a level and save a frame (no window).
fn screenshot(o: &Options, path: &str) {
    let device = MTLCreateSystemDefaultDevice().expect("Metal device");
    let mut r = Renderer::new(device);
    let mut g = make_game(o);
    let (w, h) = ((o.width * o.scale) as usize, (o.height * o.scale) as usize);
    g.view_width = o.width;
    g.view_height = o.height;
    g.scale_factor = o.scale;
    g.quality.max_pr = o.scale.min(2.0);
    g.quality.pr = g.quality.max_pr;
    g.boot();
    if let Some(s) = o.seed {
        g.run_seed = s;
    }
    if let Some(l) = o.level {
        g.init_level(l);
    } else if o.seed.is_some() {
        g.init_level(1);
    }
    if o.play {
        g.state = "playing".into();
        g.hide_overlay();
    }
    let target = r.offscreen_target(w, h);
    let t0 = Instant::now();
    for i in 0..o.frames {
        g.now_ms = (i as f64) * 1000.0 / 60.0;
        if let Some((kind, dist)) = &o.showcase {
            if g.state == "playing" && (!o.showcase_walk || i == 0) {
                g.showcase_entity(kind, *dist);
            }
            if o.death && i + 15 == o.frames {
                g.director.killer = g.entities.iter().position(|e| &e.kind == kind);
                g.on_death("entity");
            }
        }
        if let Some(s) = o.screen.as_deref().filter(|_| i + 45 == o.frames) {
            g.debug_screen(s);
        }
        g.frame(1.0 / 60.0);
        let cb = draw(&mut g, &mut r, &target, w, h);
        cb.commit();
        if i + 1 == o.frames {
            cb.waitUntilCompleted();
        }
    }
    let (tw, th, px) = r.read_texture(&target);
    engine::png::write_rgba(path, tw as u32, th as u32, &px).expect("write png");
    println!(
        "wrote {path} ({tw}x{th}) after {} frames in {:.2}s — draw calls {}, triangles {}, lights {}, programs {}",
        o.frames,
        t0.elapsed().as_secs_f64(),
        r.info.calls,
        r.info.triangles,
        r.info.lights,
        r.info.programs
    );
}

/// `--dump-level DIR`: writes a JSON description of each level — start a run
/// from `#seed=S`, then per level: trace the random draws, load it, write
/// `rs_<seed>_L<n>.json`; with `--screenshot`, rebuild it, take one fixed
/// 1/60 s step, freeze, and save `rs_<seed>_L<n>.png`.
fn level_check(o: &Options, dir: &str) {
    let seed = o.seed.unwrap_or(4242);
    let mut g = make_game(o);
    g.launch_flags = if o.flags.is_empty() { format!("seed={seed}") } else { format!("seed={seed}&{}", o.flags) };
    let (w, h) = ((o.width * o.scale) as usize, (o.height * o.scale) as usize);
    g.view_width = o.width;
    g.view_height = o.height;
    g.scale_factor = o.scale;
    g.quality.max_pr = o.scale.min(2.0);
    g.quality.pr = g.quality.max_pr;
    let device = MTLCreateSystemDefaultDevice().expect("Metal device");
    let mut r = Renderer::new(device);
    g.boot();
    g.start_new_game(); // as New Game from the menu
                        // let the film grain (repainted every 55 ms, opacity eased) settle as if the
                        // game had been running for a second
    for _ in 0..60 {
        g.update_grain(1.0 / 60.0);
    }
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
            g.fixed_step_dt = 1.0 / 60.0;
            g.frame(1.0 / 60.0);
            g.fixed_step_dt = 0.0;
            g.sim_frozen = true;
            let cb = draw(&mut g, &mut r, t, w, h);
            cb.commit();
            cb.waitUntilCompleted();
            let (tw, th, px) = r.read_texture(t);
            engine::png::write_rgba(&format!("{base}.png"), tw as u32, th as u32, &px).expect("write png");
            g.sim_frozen = false;
            print!(" (+png)");
            if o.views {
                // one frame from inside each zone (the creatures stay where they are)
                let views = g.zone_views();
                for &(zi, x, z, yaw) in &views {
                    g.player.pos = engine::math::Vec3::new(x, g.floor_y_at(x, z) + game::prelude::EYE, z);
                    g.player.yaw = yaw;
                    g.player.pitch = 0.0;
                    // the props' cull runs a few times a second; run it for this view
                    g.prop_cull.t = 0.0;
                    g.fixed_step_dt = 1.0 / 60.0;
                    g.frame(1.0 / 60.0);
                    g.fixed_step_dt = 0.0;
                    g.sim_frozen = true;
                    let cb = draw(&mut g, &mut r, t, w, h);
                    cb.commit();
                    cb.waitUntilCompleted();
                    let (tw, th, px) = r.read_texture(t);
                    engine::png::write_rgba(&format!("{base}_z{zi:02}.png"), tw as u32, th as u32, &px).expect("write png");
                    g.sim_frozen = false;
                }
                print!(" (+{} zone views)", views.len());
                // and the way down, as you would come on it
                if let Some((x, z, yaw)) = g.exit_view() {
                    g.player.pos = engine::math::Vec3::new(x, g.floor_y_at(x, z) + game::prelude::EYE, z);
                    g.player.yaw = yaw;
                    g.player.pitch = 0.0;
                    // the props' cull runs a few times a second; run it for this view
                    g.prop_cull.t = 0.0;
                    g.fixed_step_dt = 1.0 / 60.0;
                    g.frame(1.0 / 60.0);
                    g.fixed_step_dt = 0.0;
                    g.sim_frozen = true;
                    let cb = draw(&mut g, &mut r, t, w, h);
                    cb.commit();
                    cb.waitUntilCompleted();
                    let (tw, th, px) = r.read_texture(t);
                    engine::png::write_rgba(&format!("{base}_exit.png"), tw as u32, th as u32, &px).expect("write png");
                    g.sim_frozen = false;
                    print!(" (+exit view)");
                }
                // and her camp and her trail
                for (name, x, z, yaw, pitch) in g.story_views() {
                    g.player.pos = engine::math::Vec3::new(x, g.floor_y_at(x, z) + game::prelude::EYE, z);
                    g.player.yaw = yaw;
                    g.player.pitch = pitch;
                    // the props' cull runs a few times a second; run it for this view
                    g.prop_cull.t = 0.0;
                    g.fixed_step_dt = 1.0 / 60.0;
                    g.frame(1.0 / 60.0);
                    g.fixed_step_dt = 0.0;
                    g.sim_frozen = true;
                    let cb = draw(&mut g, &mut r, t, w, h);
                    cb.commit();
                    cb.waitUntilCompleted();
                    let (tw, th, px) = r.read_texture(t);
                    engine::png::write_rgba(&format!("{base}_{name}.png"), tw as u32, th as u32, &px).expect("write png");
                    g.sim_frozen = false;
                    print!(" (+{name} view)");
                }
                let lean: Vec<String> = g.lean_views(8).iter().map(|(x, z, yaw)| format!("{x:.3},{z:.3},{yaw:.4}")).collect();
                print!(" ({} leaning, lean views: \"{}\")", g.angled.len(), lean.join(";"));
                let vend: Vec<String> = g.vending_views().iter().map(|(x, z, yaw)| format!("{x:.3},{z:.3},{yaw:.4}")).collect();
                print!(" (vending views: \"{}\", {})", vend.join(";"), g.vending_states().join(" "));
            }
            if let Some(what) = o.vending_demo.as_deref() {
                g.vending_demo(what);
            }
            for (k, &(x, z, yaw)) in o.views_at.iter().enumerate() {
                g.player.pos = engine::math::Vec3::new(x, g.floor_y_at(x, z) + game::prelude::EYE, z);
                g.player.yaw = yaw;
                g.player.pitch = 0.0;
                g.prop_cull.t = 0.0;
                g.fixed_step_dt = 1.0 / 60.0;
                g.frame(1.0 / 60.0);
                g.fixed_step_dt = 0.0;
                g.sim_frozen = true;
                let cb = draw(&mut g, &mut r, t, w, h);
                cb.commit();
                cb.waitUntilCompleted();
                let (tw, th, px) = r.read_texture(t);
                engine::png::write_rgba(&format!("{base}_v{k:02}.png"), tw as u32, th as u32, &px).expect("write png");
                g.sim_frozen = false;
            }
        }
        println!();
    }
}

/// `--metrics FILE`: start a run like New Game, build each level in order and
/// write their `level_metrics` as one JSON array.
fn metrics(o: &Options, path: &str) {
    let seed = o.seed.unwrap_or(4242);
    let mut g = make_game(o);
    g.launch_flags = if o.flags.is_empty() { format!("seed={seed}&nosound") } else { format!("seed={seed}&nosound&{}", o.flags) };
    g.boot();
    g.start_new_game();
    let levels = if o.levels.is_empty() { vec![o.level.unwrap_or(1)] } else { o.levels.clone() };
    let mut out = vec![];
    for &l in &levels {
        g.load_level(l);
        out.push(g.level_metrics());
    }
    std::fs::write(path, serde_json::Value::Array(out).to_string()).expect("write metrics");
}

/// The `p`-quantile of `v` (sorted in place).
fn quantile(v: &mut [f64], p: f64) -> f64 {
    v.sort_by(|a, b| a.total_cmp(b));
    if v.is_empty() {
        0.0
    } else {
        v[((v.len() - 1) as f64 * p).round() as usize]
    }
}

/// `--bench FILE`: for each level, the time to build it, to load plus render
/// its first frame (pipeline compiles land there), and per frame along a
/// fixed path through its zones (each zone view, turning slowly) the game
/// update on the CPU and the frame on the GPU (`MTLCommandBuffer` GPU start
/// and end), with draw calls and triangles. Written as one JSON array.
fn bench(o: &Options, path: &str) {
    let seed = o.seed.unwrap_or(4242);
    let mut g = make_game(o);
    g.launch_flags = if o.flags.is_empty() { format!("seed={seed}&nosound") } else { format!("seed={seed}&nosound&{}", o.flags) };
    let (w, h) = ((o.width * o.scale) as usize, (o.height * o.scale) as usize);
    g.view_width = o.width;
    g.view_height = o.height;
    g.scale_factor = o.scale;
    g.quality.max_pr = o.scale.min(2.0);
    g.quality.pr = g.quality.max_pr;
    let device = MTLCreateSystemDefaultDevice().expect("Metal device");
    let mut r = Renderer::new(device);
    let target = r.offscreen_target(w, h);
    g.boot();
    g.start_new_game();
    g.state = "playing".into();
    g.hide_overlay();
    let levels = if o.levels.is_empty() { vec![o.level.unwrap_or(1)] } else { o.levels.clone() };
    let mut out = vec![];
    for &l in &levels {
        let t0 = Instant::now();
        g.load_level(l);
        let build_ms = t0.elapsed().as_secs_f64() * 1000.0;
        let t1 = Instant::now();
        g.fixed_step_dt = 1.0 / 60.0;
        g.frame(1.0 / 60.0);
        let cb = draw(&mut g, &mut r, &target, w, h);
        cb.commit();
        cb.waitUntilCompleted();
        let first_ms = t1.elapsed().as_secs_f64() * 1000.0;
        let views = g.zone_views();
        let per_view = (o.bench_frames / views.len().max(1)).max(1);
        let (mut upd, mut gpu, mut calls, mut tris) = (vec![], vec![], vec![], vec![]);
        for k in 0..o.bench_frames {
            if let Some(&(_, x, z, yaw)) = views.get((k / per_view) % views.len().max(1)) {
                g.player.pos = engine::math::Vec3::new(x, g.floor_y_at(x, z) + game::prelude::EYE, z);
                g.player.yaw = yaw + (k % per_view) as f64 * 0.01;
                g.player.pitch = 0.0;
            }
            g.now_ms += 1000.0 / 60.0;
            let tu = Instant::now();
            g.frame(1.0 / 60.0);
            upd.push(tu.elapsed().as_secs_f64() * 1000.0);
            let cb = draw(&mut g, &mut r, &target, w, h);
            cb.commit();
            cb.waitUntilCompleted();
            gpu.push((cb.GPUEndTime() - cb.GPUStartTime()) * 1000.0);
            calls.push(r.info.calls as f64);
            tris.push(r.info.triangles as f64);
        }
        g.fixed_step_dt = 0.0;
        // the frames whose update took over 2 ms (which, and how long): what
        // a high p99 is made of
        let slow: Vec<serde_json::Value> =
            upd.iter().enumerate().filter(|(_, v)| **v > 2.0).map(|(i, v)| serde_json::json!([i, (v * 100.0).round() / 100.0])).collect();
        let r3 = |x: f64| (x * 1000.0).round() / 1000.0;
        let calls_avg = calls.iter().sum::<f64>() / calls.len().max(1) as f64;
        out.push(serde_json::json!({
            "seed": seed, "level": l, "frames": o.bench_frames, "size": [w, h],
            "build_ms": r3(build_ms), "first_frame_ms": r3(first_ms),
            "update_p50_ms": r3(quantile(&mut upd, 0.5)), "update_p99_ms": r3(quantile(&mut upd, 0.99)), "update_slow": slow,
            "gpu_p50_ms": r3(quantile(&mut gpu, 0.5)), "gpu_p99_ms": r3(quantile(&mut gpu, 0.99)),
            "calls_avg": r3(calls_avg), "calls_max": quantile(&mut calls, 1.0), "tris_max": quantile(&mut tris, 1.0),
        }));
    }
    std::fs::write(path, serde_json::Value::Array(out).to_string()).expect("write bench");
}

/// `--bench-creatures FILE`: the creatures' cost. On level 4 of the run, a
/// pack of every kind that roams the rooms (two of each) is placed round you
/// and set hunting you (god mode); then, over `bench_frames` frames while
/// you turn once on the spot, the game update on the CPU, the frame on the
/// GPU, draw calls and triangles. Written as one JSON object.
fn bench_creatures(o: &Options, path: &str) {
    let seed = o.seed.unwrap_or(4242);
    let mut g = make_game(o);
    g.launch_flags = format!("seed={seed}&nosound");
    let (w, h) = ((o.width * o.scale) as usize, (o.height * o.scale) as usize);
    g.view_width = o.width;
    g.view_height = o.height;
    g.scale_factor = o.scale;
    g.quality.max_pr = o.scale.min(2.0);
    g.quality.pr = g.quality.max_pr;
    let device = MTLCreateSystemDefaultDevice().expect("Metal device");
    let mut r = Renderer::new(device);
    let target = r.offscreen_target(w, h);
    g.boot();
    g.start_new_game();
    g.state = "playing".into();
    g.hide_overlay();
    g.load_level(4);
    g.fixed_step_dt = 1.0 / 60.0;
    let placed = g.bench_pack(2);
    // a few frames to settle (pipelines compile, the notices play out)
    for _ in 0..30 {
        g.now_ms += 1000.0 / 60.0;
        g.frame(1.0 / 60.0);
        let cb = draw(&mut g, &mut r, &target, w, h);
        cb.commit();
        cb.waitUntilCompleted();
    }
    let (mut upd, mut gpu, mut calls, mut tris) = (vec![], vec![], vec![], vec![]);
    let yaw0 = g.player.yaw;
    for k in 0..o.bench_frames {
        g.player.yaw = yaw0 + k as f64 / o.bench_frames as f64 * std::f64::consts::TAU;
        g.now_ms += 1000.0 / 60.0;
        let tu = Instant::now();
        g.frame(1.0 / 60.0);
        upd.push(tu.elapsed().as_secs_f64() * 1000.0);
        let cb = draw(&mut g, &mut r, &target, w, h);
        cb.commit();
        cb.waitUntilCompleted();
        gpu.push((cb.GPUEndTime() - cb.GPUStartTime()) * 1000.0);
        calls.push(r.info.calls as f64);
        tris.push(r.info.triangles as f64);
    }
    let r3 = |x: f64| (x * 1000.0).round() / 1000.0;
    let out = serde_json::json!({
        "seed": seed, "level": 4, "creatures": placed, "frames": o.bench_frames, "size": [w, h],
        "update_p50_ms": r3(quantile(&mut upd, 0.5)), "update_p99_ms": r3(quantile(&mut upd, 0.99)),
        "gpu_p50_ms": r3(quantile(&mut gpu, 0.5)), "gpu_p99_ms": r3(quantile(&mut gpu, 0.99)),
        "calls_max": quantile(&mut calls, 1.0), "tris_max": quantile(&mut tris, 1.0),
    });
    std::fs::write(path, out.to_string()).expect("write bench");
}

fn main() {
    let o = parse_args();
    if let Some(p) = o.bench.clone() {
        bench(&o, &p);
        return;
    }
    if let Some(p) = o.bench_creatures.clone() {
        bench_creatures(&o, &p);
        return;
    }
    if let Some(p) = o.metrics.clone() {
        metrics(&o, &p);
        return;
    }
    if let Some(d) = o.dump_level.clone() {
        level_check(&o, &d);
        return;
    }
    if let Some(p) = o.screenshot.clone() {
        screenshot(&o, &p);
        return;
    }
    // a recorded session seeds the per-frame randomness (and stores the
    // seed); a replay seeds it the same way, before the game is made
    let mut replaying = None;
    let mut recorder = None;
    if let Some(p) = &o.replay {
        let text = std::fs::read_to_string(p).unwrap_or_else(|e| panic!("{p}: {e}"));
        let rec = game::debug::replay::Recording::from_json(&serde_json::from_str(&text).expect("a recording"));
        game::base::rng::seed_native(rec.native_seed);
        println!("replaying {p}: {} frames; Up / Down arrows mark tension rising / falling, Escape stops", rec.frames.len());
        replaying = Some(Replaying { rec, next: 0, trace: format!("{p}.trace.json"), marks: vec![] });
    } else if let Some(p) = &o.record {
        let seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(1);
        game::base::rng::seed_native(seed);
        println!("recording this session to {p} (on this machine only)");
        recorder = Some((game::debug::replay::Recorder::new(seed), p.clone()));
    }
    let g = make_game(&o);
    run(
        AppConfig { title: "The Backrooms".into(), width: o.width, height: o.height, fullscreen: o.fullscreen },
        App { g, started: Instant::now(), cursor: (0.0, 0.0), last_hit: String::new(), recorder, replaying, frames: 0 },
    );
}
