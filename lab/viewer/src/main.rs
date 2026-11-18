//! The creature lab: the creatures' motion and the generated plants, shown
//! apart from the game (`lab/README.md`). Nothing here is read by the game.
//!
//! ```sh
//! cargo run --release                          # the window (keys: window.rs)
//! cargo run --release -- --plants DIR [--seed N]
//! cargo run --release -- --scene DIR [--view side|eye] [--lamp] [--wrong W] [--every S] [--no-video]
//! cargo run --release -- --clips DIR           # each clip in place, a sheet each
//! cargo run --release -- --metrics [DIR]       # the motion's numbers, the game's and the new
//! cargo run --release -- --motion FILE …       # another clip library (default: lab/motion/clips/smiler.motion)
//! ```
mod legacy;
mod motion;
mod plants;
mod smiler;
mod stage;
mod window;

use engine::prelude::*;
use legacy::Legacy;
use motion::{Gains, Library};
use plants::Species;
use smiler::{SCENE, Smiler};
use stage::{Light, Orbit, Stage, sheet, write_png};

fn arg(args: &[String], name: &str) -> Option<String> {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned()
}

fn flag(args: &[String], name: &str) -> bool {
    args.iter().any(|a| a == name)
}

fn renderer() -> engine::render::Renderer {
    let device = objc2_metal::MTLCreateSystemDefaultDevice().expect("a Metal device");
    engine::render::Renderer::new(device)
}

const LIBRARY: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../motion/clips/smiler.motion");

/// The gallery: each species tended, left a season and left for years, a
/// row each; and each plant close, from two sides.
fn plants_offscreen(dir: &str, seed: u64) {
    let mut r = renderer();
    let (w, h) = (960, 600);
    let stage = Stage::new(w as f64 / h as f64);
    let healths = [1.0, 0.45, 0.0];
    let mut close = vec![];
    for (si, s) in Species::ALL.iter().enumerate() {
        for (hi, &hh) in healths.iter().enumerate() {
            let p = s.grow(seed + si as u64, hh);
            println!("{:<14} health {hh:.2} ({}): {:>6} triangles", s.name(), plants::health_word(hh), p.triangles());
            let node = plants::to_scene(&p);
            stage.scene.add(&node);
            let top = if *s == Species::Ficus { 0.8 } else { 0.32 };
            let dist = if *s == Species::Ficus { 2.6 } else { 1.25 };
            let mut frames = vec![];
            for yaw in [0.5, 2.3] {
                stage.look(&Orbit { target: [0.0, top, 0.0], yaw, pitch: 0.28, dist });
                frames.push(stage.render(&mut r, w / 2, h / 2));
            }
            stage.scene.remove(&node);
            let name = s.name().replace(' ', "_");
            write_png(&format!("{dir}/{name}_{hi}.png"), &sheet(&frames, 2));
            close.extend(frames.into_iter().take(1));
        }
    }
    write_png(&format!("{dir}/gallery.png"), &sheet(&close, 3));
    println!("wrote {dir}/gallery.png (rows: species; columns: tended, left a season, left for years)");
}

fn load(args: &[String]) -> Library {
    let path = arg(args, "--motion").unwrap_or(LIBRARY.into());
    Library::load(&path).unwrap_or_else(|e| {
        eprintln!("{e}\n(make it: cd lab/motion && .venv/bin/python motion.py gen && .venv/bin/python motion.py bake)");
        std::process::exit(2)
    })
}

/// Where the viewer stands in the scene.
const EYE: (f64, f64, f64) = (0.4, 1.6, 7.6);

/// The scene's length at most (a steered beat ends when it arrives).
fn scene_len() -> f64 {
    SCENE.iter().map(|b| b.secs).sum()
}

/// The camera: from the viewer's eye, or from the side, following the body.
/// The viewer's eye during beat `beat`: standing, or on the floor once the
/// Smiler has struck (it stands over you and looks down).
fn eye_at(beat: usize) -> Vec3 {
    let down = SCENE[beat].mode == smiler::Mode::Over;
    v3(EYE.0, if down { 0.35 } else { EYE.1 }, EYE.2)
}

fn camera(view: &str, at: Vec3, eye: Vec3) -> Orbit {
    if view == "eye" {
        let (dx, dz) = (at.x - eye.x, at.z - eye.z);
        let yaw = dx.datan2(dz) + PI;
        let d = dx.dhypot(dz).max(0.5);
        Orbit { target: [at.x, 1.2, at.z], yaw, pitch: ((eye.y - 1.2) / d).datan(), dist: d.dhypot(eye.y - 1.2) }
    } else {
        Orbit { target: [at.x, 1.05, at.z], yaw: 0.95, pitch: 0.12, dist: 4.6 }
    }
}

/// The scene, old and new side by side: frames at 30 fps (a video), and
/// sheets at `every` seconds.
fn scene_offscreen(args: &[String], dir: &str) {
    let lib = load(args);
    let view = arg(args, "--view").unwrap_or("side".into());
    let every: f64 = arg(args, "--every").and_then(|s| s.parse().ok()).unwrap_or(1.0);
    let mut r = renderer();
    let (w, h) = (640, 480);
    let mut stage = Stage::new(w as f64 / h as f64);
    if flag(args, "--lamp") {
        stage.set_light(Light::Lamp);
    }
    let mut new = Smiler::new(&lib).expect("the Smiler's body and a humanoid map");
    if let Some(wr) = arg(args, "--wrong").and_then(|s| s.parse().ok()) {
        new.wrong = wr;
    }
    let mut old = Legacy::new().expect("the Smiler's body");
    stage.scene.add(&new.rig.root);
    stage.scene.add(&old.rig.root);
    let fps = 30.0;
    let n = (scene_len() * fps) as usize;
    let frames_dir = format!("{dir}/frames");
    let _ = std::fs::create_dir_all(&frames_dir);
    let video = !flag(args, "--no-video");
    let (mut sheet_old, mut sheet_new) = (vec![], vec![]);
    // a strip a beat, a frame every 0.2 s: the blind review's pairs
    let mut strips: Vec<(String, Vec<_>, Vec<_>)> = vec![];
    let (mut mode_t, mut last) = (0.0, None);
    let mut prev = new.pos;
    println!("the scene: {:.1} s, {n} frames, the k of its legs over the mocap's {:.2}", scene_len(), new.rt.k);
    for f in 0..n {
        let dt = 1.0 / fps;
        let eye = eye_at(new.beat);
        let was = new.beat;
        new.update(&lib, dt, eye);
        // once round: the scene is over when it starts again
        if new.beat < was {
            break;
        }
        let mode = new.mode();
        if Some(mode) != last {
            mode_t = 0.0;
            last = Some(mode);
        }
        mode_t += dt;
        let (vx, vz) = ((new.pos.x - prev.x) / dt, (new.pos.z - prev.z) / dt);
        prev = new.pos;
        old.animate(new.pos, new.yaw, vx, vz, mode, mode_t, eye, new.time, dt);
        let cam = camera(&view, new.pos, eye);
        stage.look(&cam);
        let t = f as f64 / fps;
        let keep = (t / every).fract() < 0.5 / (every * fps);
        let name = format!("{:02}_{}", new.beat, SCENE[new.beat].clip);
        if strips.last().is_none_or(|s| s.0 != name) {
            strips.push((name, vec![], vec![]));
        }
        let strip = ((new.beat_t * fps).round() as usize).is_multiple_of(6) && strips.last().is_some_and(|s| s.1.len() < 10);
        if !video && !keep && !strip {
            continue;
        }
        new.rig.root.set_visible(false);
        old.rig.root.set_visible(true);
        let a = stage.render(&mut r, w, h);
        old.rig.root.set_visible(false);
        new.rig.root.set_visible(true);
        let b = stage.render(&mut r, w, h);
        if strip && let Some(s) = strips.last_mut() {
            s.1.push(a.clone());
            s.2.push(b.clone());
        }
        if keep {
            sheet_old.push(a.clone());
            sheet_new.push(b.clone());
            println!("{t:5.1} s  {:?} ({})", mode, SCENE[new.beat].clip);
        }
        if video {
            write_png(&format!("{frames_dir}/{f:04}.png"), &sheet(&[a, b], 2));
        }
    }
    for (name, a, b) in &strips {
        if !a.is_empty() {
            write_png(&format!("{dir}/strips_old/{name}.png"), &sheet(a, 5));
            write_png(&format!("{dir}/strips_new/{name}.png"), &sheet(b, 5));
        }
    }
    write_png(&format!("{dir}/sheet_old.png"), &sheet(&sheet_old, 6));
    write_png(&format!("{dir}/sheet_new.png"), &sheet(&sheet_new, 6));
    let pairs: Vec<_> = sheet_old.iter().zip(&sheet_new).flat_map(|(a, b)| [a.clone(), b.clone()]).collect();
    write_png(&format!("{dir}/sheet_pairs.png"), &sheet(&pairs, 4));
    if video {
        let out = format!("{dir}/scene_{view}.mp4");
        let ok = std::process::Command::new("ffmpeg")
            .args([
                "-y",
                "-loglevel",
                "error",
                "-framerate",
                "30",
                "-i",
                &format!("{frames_dir}/%04d.png"),
                "-c:v",
                "libx264",
                "-pix_fmt",
                "yuv420p",
                "-crf",
                "20",
                &out,
            ])
            .status()
            .is_ok_and(|s| s.success());
        println!(
            "{}",
            if ok {
                format!("wrote {out} (left: the game's animation; right: the new)")
            } else {
                "ffmpeg failed: the frames are in frames/".into()
            }
        );
    }
    println!("wrote {dir}/sheet_pairs.png (each pair: the game's, then the new)");
}

/// Each clip on the body in place: a sheet each, a frame every 0.25 s.
fn clips_offscreen(args: &[String], dir: &str) {
    let lib = load(args);
    let mut r = renderer();
    let (w, h) = (360, 420);
    let stage = Stage::new(w as f64 / h as f64);
    let s = Smiler::new(&lib).expect("the Smiler's body");
    stage.scene.add(&s.rig.root);
    s.rig.root.set_pos(0, 0, 0);
    s.rig.root.update_matrix_world(true);
    for c in &lib.clips {
        let mut frames = vec![];
        let len = c.duration(lib.fps).min(6.0);
        let mut t = 0.0;
        while t < len {
            let fr = c.sample(t, lib.fps);
            s.rt.pose(&s.rig, &fr.p, &Gains::default(), [None, None]);
            s.rig.sync();
            stage.look(&Orbit { target: [0.0, 1.05, 0.0], yaw: 1.15, pitch: 0.1, dist: 4.0 });
            frames.push(stage.render(&mut r, w, h));
            t += 0.25;
        }
        write_png(&format!("{dir}/{}.png", c.name), &sheet(&frames, 8));
        println!("{:<12} {:>4} frames {:5.2} s  speed {:.2} m/s  “{}”", c.name, c.frames.len(), c.duration(lib.fps), c.speed, c.prompt);
    }
}

/// The motion's numbers over the scene, the game's animation and the new:
/// how far a planted foot slides, how smooth the body moves, how much of it
/// moves at all, and how far the feet go through the floor.
fn metrics(args: &[String]) {
    let lib = load(args);
    let mut new = Smiler::new(&lib).expect("the Smiler's body");
    let mut old = Legacy::new().expect("the Smiler's body");
    let fps = 30.0;
    let n = (scene_len() * fps) as usize;
    let (mut mode_t, mut last) = (0.0, None);
    let mut prev = new.pos;
    let names = [
        "hips",
        "spine",
        "chest",
        "neck",
        "head",
        "upperarm.L",
        "forearm.L",
        "hand.L",
        "upperarm.R",
        "forearm.R",
        "hand.R",
        "shin.L",
        "foot.L",
        "shin.R",
        "foot.R",
        "toes.L",
        "toes.R",
    ];
    let world = |rig: &game::entities::body::BodyRig, i: usize| {
        let p = rig.head(i);
        let (pos, yaw) = (rig.root.pos(), rig.root.rotation().y);
        let (s, c) = yaw.dsin_cos();
        v3(pos.x + p.x * c + p.z * s, p.y, pos.z - p.x * s + p.z * c)
    };
    let mut tracks: [Vec<Vec<Vec3>>; 2] = [vec![], vec![]];
    let mut beats: Vec<usize> = vec![];
    let mut quats: [Vec<Vec<Quat>>; 2] = [vec![], vec![]];
    for _ in 0..n {
        let dt = 1.0 / fps;
        let eye = eye_at(new.beat);
        let was = new.beat;
        new.update(&lib, dt, eye);
        // once round: the scene is over when it starts again
        if new.beat < was {
            break;
        }
        let mode = new.mode();
        if Some(mode) != last {
            mode_t = 0.0;
            last = Some(mode);
        }
        mode_t += dt;
        let (vx, vz) = ((new.pos.x - prev.x) / dt, (new.pos.z - prev.z) / dt);
        prev = new.pos;
        old.animate(new.pos, new.yaw, vx, vz, mode, mode_t, eye, new.time, dt);
        beats.push(new.beat);
        if std::env::var("LAB_TRACE").is_ok_and(|b| b == SCENE[new.beat].clip) {
            let foot = |r: &game::entities::body::BodyRig, n: &str| world(r, r.ix(n).unwrap());
            let (l, rr) = (foot(&new.rig, "foot.L"), foot(&new.rig, "foot.R"));
            let k = new.locks();
            println!(
                "t {:5.2} pos {:+.2} {:+.2}  L lock {} gap {:.2} at {:+.2} {:+.2} {:.2}  R lock {} gap {:.2} at {:+.2} {:+.2} {:.2}",
                new.time, new.pos.x, new.pos.z, k[0].0 as u8, k[0].1, l.x, l.z, l.y, k[1].0 as u8, k[1].1, rr.x, rr.z, rr.y
            );
        }
        for (k, rig) in [&old.rig, &new.rig].into_iter().enumerate() {
            tracks[k].push(names.iter().map(|nm| world(rig, rig.ix(nm).unwrap())).collect());
            quats[k].push(names.iter().map(|nm| rig.body_rot(rig.ix(nm).unwrap())).collect());
        }
    }
    let mut out = String::from("| | the game's | the new |\n|---|---|---|\n");
    let mut row = |label: &str, f: &dyn Fn(usize) -> f64, unit: &str| {
        out += &format!("| {label} | {:.3} {unit} | {:.3} {unit} |\n", f(0), f(1));
    };
    // what touches the floor: each foot's ankle and ball, on the floor when
    // within 1.5 cm of the lowest it goes in the scene
    let contact_pts = [12usize, 14, 15, 16];
    let slide_in = |k: usize, keep: &dyn Fn(usize) -> bool| {
        let tr = &tracks[k];
        let (mut sum, mut cnt) = (0.0f64, 0.0f64);
        for &i in &contact_pts {
            let floor = tr.iter().map(|f| f[i].y).fold(f64::MAX, f64::min);
            for t in 1..tr.len() {
                if keep(t) && tr[t][i].y < floor + 0.015 && tr[t - 1][i].y < floor + 0.015 {
                    sum += (tr[t][i].x - tr[t - 1][i].x).dhypot(tr[t][i].z - tr[t - 1][i].z) * fps;
                    cnt += 1.0;
                }
            }
        }
        sum / cnt.max(1.0)
    };
    let skate = |k: usize| slide_in(k, &|_| true);
    row("a foot sliding while it touches the floor (ankle or ball within 1.5 cm of its lowest), mean", &skate, "m/s");
    // the same, beat by beat
    let mut by_beat = String::from("\n| beat | the game's | the new |\n|---|---|---|\n");
    for (bi, b) in SCENE.iter().enumerate() {
        let slide = |k: usize| slide_in(k, &|t| beats[t] == bi && beats[t - 1] == bi);
        by_beat += &format!("| {:?} ({}) | {:.3} m/s | {:.3} m/s |\n", b.mode, b.clip, slide(0), slide(1));
    }
    let jerk = |k: usize| {
        let tr = &tracks[k];
        let (mut sum, mut cnt) = (0.0, 0.0);
        for t in 3..tr.len() {
            for i in 0..names.len() {
                let d = |a: Vec3, b: Vec3| v3(a.x - b.x, a.y - b.y, a.z - b.z);
                let j3 = d(
                    d(d(tr[t][i], tr[t - 1][i]), d(tr[t - 1][i], tr[t - 2][i])),
                    d(d(tr[t - 1][i], tr[t - 2][i]), d(tr[t - 2][i], tr[t - 3][i])),
                );
                sum += j3.length() * fps * fps * fps;
                cnt += 1.0;
            }
        }
        sum / cnt
    };
    row("jerk of the body's joints, mean (smoothness: lower is smoother)", &jerk, "m/s³");
    let alive = |k: usize| {
        let q = &quats[k];
        let (mut sum, mut cnt) = (0.0, 0.0);
        for t in 1..q.len() {
            for i in 0..names.len() {
                let d = q[t][i].dot(&q[t - 1][i]).abs().min(1.0);
                sum += 2.0 * d.dacos() * fps;
                cnt += 1.0;
            }
        }
        sum / cnt
    };
    row("how much the body turns at its joints, mean", &alive, "rad/s");
    let still = |k: usize| {
        // the same, while it stands in its dark (stillness is a vocabulary)
        let q = &quats[k];
        let m = (SCENE[0].secs * fps) as usize;
        let (mut sum, mut cnt) = (0.0, 0.0);
        for t in 1..m {
            for i in 0..names.len() {
                sum += 2.0 * q[t][i].dot(&q[t - 1][i]).abs().min(1.0).dacos() * fps;
                cnt += 1.0;
            }
        }
        sum / cnt
    };
    row("the same, standing in its dark (Idle)", &still, "rad/s");
    let below = |k: usize| tracks[k].iter().flat_map(|f| [f[12].y, f[14].y]).fold(f64::MAX, f64::min);
    row("lowest ankle over the scene", &below, "m");
    out += &by_beat;
    println!("{out}");
    if let Some(dir) = arg(args, "--metrics").filter(|d| !d.starts_with("--")) {
        let _ = std::fs::create_dir_all(&dir);
        std::fs::write(format!("{dir}/metrics.md"), &out).expect("write metrics");
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let seed = arg(&args, "--seed").and_then(|s| s.parse().ok()).unwrap_or(7);
    if flag(&args, "--help") {
        println!("{}", include_str!("main.rs").lines().take_while(|l| l.starts_with("//!")).collect::<Vec<_>>().join("\n"));
        return;
    }
    if let Some(dir) = arg(&args, "--plants") {
        plants_offscreen(&dir, seed);
        return;
    }
    if let Some(dir) = arg(&args, "--scene") {
        scene_offscreen(&args, &dir);
        return;
    }
    if let Some(dir) = arg(&args, "--clips") {
        clips_offscreen(&args, &dir);
        return;
    }
    if flag(&args, "--metrics") {
        metrics(&args);
        return;
    }
    let path = arg(&args, "--motion").unwrap_or(LIBRARY.into());
    let lib = match Library::load(&path) {
        Ok(l) => Some(l),
        Err(e) => {
            eprintln!("{e}: the plants only (make the clips with lab/motion/motion.py)");
            None
        }
    };
    engine::app::run(
        engine::app::AppConfig { title: "creature lab".into(), width: 1280.0, height: 800.0, fullscreen: false },
        window::Lab::new(lib, seed),
    );
}
