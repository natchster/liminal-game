//! The creature lab: every creature's motion, as the game plays it, and the
//! generated plants, shown apart from the game (`lab/README.md`).
//!
//! The creatures move by the game's own code (`entities::motion` and each
//! kind's `pose`): what is seen here is what the game does. Beside each the
//! game's animation from before its clips can walk the same path (`legacy`).
//!
//! ```sh
//! cargo run --release                          # the window (keys: window.rs)
//! cargo run --release -- --plants DIR [--seed N]
//! cargo run --release -- --scene DIR [--creature K|all] [--view side|eye] [--lamp] [--wrong W] [--every S] [--no-video]
//! cargo run --release -- --lineup DIR          # all six through their scenes, side by side
//! cargo run --release -- --clips DIR [--creature K|all] [--bridges]   # each clip on its body, in place (and its bridges)
//! cargo run --release -- --metrics [DIR] [--creature K|all]   # the numbers, the old animation against the new
//! cargo run --release -- --motion DIR …        # the libraries in DIR (e.g. motion/clips, from lab/: a bake not yet installed)
//! cargo run --release -- --scene|--metrics … --against DIR   # beside it the same creature on DIR's libraries, not the old animation
//! ```
mod creatures;
mod legacy;
mod plants;
mod stage;
mod window;

use creatures::{KINDS, Specimen};
use engine::prelude::*;
use game::entities::body::BodyRig;
use game::entities::motion::Library;
use plants::Species;
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

/// The libraries the lab shows: `--motion DIR`'s `<kind>.motion` where there
/// is one, else the game's own (`None`).
pub fn libraries(args: &[String]) -> Vec<Option<&'static Library>> {
    libraries_in(arg(args, "--motion"))
}

/// What a scene or the metrics set beside the creature: the same creature
/// on `--against DIR`'s libraries (one bake against another), else the old
/// animation.
fn twin(args: &[String], kind: &'static str) -> Option<Specimen> {
    let dir = arg(args, "--against")?;
    Some(specimen(kind, &libraries_in(Some(dir))))
}

fn libraries_in(dir: Option<String>) -> Vec<Option<&'static Library>> {
    // a folder that is not there is an error, never the game's own clips in
    // its place: a comparison against them proved nothing, and said "identical"
    if let Some(d) = &dir
        && !std::path::Path::new(d).is_dir()
    {
        eprintln!("{d}: no such folder of libraries (paths are from where the lab runs, lab/)");
        std::process::exit(2);
    }
    KINDS
        .iter()
        .map(|k| {
            let dir = dir.as_ref()?;
            let path = format!("{dir}/{k}.motion");
            let Ok(bytes) = std::fs::read(&path) else {
                // (the Watcher has no clips: its poses are its body's)
                if *k != "watcher" {
                    eprintln!("{path}: not there (the game's clips instead)");
                }
                return None;
            };
            match Library::parse(&bytes) {
                Ok(l) => Some(&*Box::leak(Box::new(l))),
                Err(e) => {
                    eprintln!("{path}: {e} (the game's clips instead)");
                    None
                }
            }
        })
        .collect()
}

/// The kinds `--creature` names (all, by default).
fn chosen(args: &[String]) -> Vec<&'static str> {
    match arg(args, "--creature").as_deref() {
        None | Some("all") => KINDS.to_vec(),
        Some(list) => KINDS.iter().copied().filter(|k| list.split(',').any(|w| w == *k)).collect(),
    }
}

fn specimen(kind: &'static str, libs: &[Option<&'static Library>]) -> Specimen {
    let i = KINDS.iter().position(|k| *k == kind).unwrap_or(0);
    Specimen::new(kind, libs[i]).unwrap_or_else(|| panic!("{kind}: no body, or not a humanoid one"))
}

/// The camera: from the viewer's eye, or from the side, following the body.
fn camera(view: &str, at: Vec3, eye: Vec3, kind: &str) -> Orbit {
    let h = if kind == "hound" { 0.6 } else { 1.05 };
    if view == "eye" {
        let (dx, dz) = (at.x - eye.x, at.z - eye.z);
        let yaw = dx.datan2(dz) + PI;
        let d = dx.dhypot(dz).max(0.5);
        Orbit { target: [at.x, h + 0.15, at.z], yaw, pitch: ((eye.y - h - 0.15) / d).datan(), dist: d.dhypot(eye.y - h - 0.15) }
    } else {
        Orbit { target: [at.x, h, at.z], yaw: 0.95, pitch: 0.12, dist: if kind == "hound" { 2.6 } else { 4.6 } }
    }
}

/// Each creature's scene, old and new side by side: frames at 30 fps (a
/// video), sheets at `every` seconds, and a strip a beat (a frame every
/// 0.2 s: the blind review's pairs).
fn scene_offscreen(args: &[String], dir: &str) {
    let libs = libraries(args);
    let view = arg(args, "--view").unwrap_or("side".into());
    let every: f64 = arg(args, "--every").and_then(|s| s.parse().ok()).unwrap_or(1.0);
    let video = !flag(args, "--no-video");
    let mut r = renderer();
    let (w, h) = (640, 480);
    for kind in chosen(args) {
        let mut stage = Stage::new(w as f64 / h as f64);
        if flag(args, "--lamp") {
            stage.set_light(Light::Lamp);
        }
        let mut tw = twin(args, kind);
        let mut s = if tw.is_some() { specimen(kind, &libs) } else { specimen(kind, &libs).with_legacy() };
        if let Some(wr) = arg(args, "--wrong").and_then(|s| s.parse().ok()) {
            for a in [s.anim.as_mut(), tw.as_mut().and_then(|t| t.anim.as_mut())].into_iter().flatten() {
                a.wrong = wr;
            }
        }
        stage.scene.add(&s.rig.root);
        if let Some(l) = &s.legacy {
            stage.scene.add(&l.rig.root);
        }
        if let Some(t) = &tw {
            stage.scene.add(&t.rig.root);
        }
        let out = format!("{dir}/{kind}");
        let frames_dir = format!("{out}/frames");
        let _ = std::fs::remove_dir_all(&frames_dir);
        let _ = std::fs::create_dir_all(&frames_dir);
        let fps = 30.0;
        let len: f64 = s.scene.iter().map(|b| b.secs).sum();
        let (mut sheet_old, mut sheet_new) = (vec![], vec![]);
        let mut strips: Vec<(String, Vec<_>, Vec<_>)> = vec![];
        println!("{}: the scene, at most {len:.1} s", creatures::title(kind));
        let mut f = 0usize;
        loop {
            let dt = 1.0 / fps;
            let eye = s.eye();
            if s.step(dt, eye) || f as f64 > len * fps + 1.0 {
                break;
            }
            s.pose_legacy(dt, eye, 0.0);
            if let Some(t) = tw.as_mut() {
                t.step(dt, eye);
            }
            let cam = camera(&view, s.pos, eye, kind);
            stage.look(&cam);
            let t = f as f64 / fps;
            let keep = (t / every).fract() < 0.5 / (every * fps);
            let name = format!("{:02}_{}", s.beat, s.current().label.replace([' ', ':', '(', ')', ',', '/'], "_"));
            if strips.last().is_none_or(|x| x.0 != name) {
                strips.push((name, vec![], vec![]));
            }
            let strip = ((s.beat_t * fps).round() as usize).is_multiple_of(6) && strips.last().is_some_and(|x| x.1.len() < 10);
            f += 1;
            if !video && !keep && !strip {
                continue;
            }
            let old_rig = tw.as_ref().map(|t| &t.rig).or(s.legacy.as_ref().map(|l| &l.rig));
            let has_old = old_rig.is_some();
            let a = if let Some(l) = old_rig {
                s.rig.root.set_visible(false);
                l.root.set_visible(true);
                let a = stage.render(&mut r, w, h);
                l.root.set_visible(false);
                Some(a)
            } else {
                None
            };
            s.rig.root.set_visible(true);
            let b = stage.render(&mut r, w, h);
            if strip && let Some(x) = strips.last_mut() {
                if let Some(a) = &a {
                    x.1.push(a.clone());
                }
                x.2.push(b.clone());
            }
            if keep {
                if let Some(a) = &a {
                    sheet_old.push(a.clone());
                }
                sheet_new.push(b.clone());
                let (clip, ct) = s.anim.as_ref().map_or(("poses", 0.0), |a| a.playing());
                println!("{t:5.1} s  {:<10} {:<12} {ct:5.2}  {}", s.mode.name(), clip, s.current().label);
            }
            if video {
                let pair = if has_old { sheet(&[a.unwrap(), b], 2) } else { b };
                write_png(&format!("{frames_dir}/{:04}.png", f - 1), &pair);
            }
        }
        for (name, a, b) in &strips {
            if !a.is_empty() {
                write_png(&format!("{out}/strips_old/{name}.png"), &sheet(a, 5));
            }
            if !b.is_empty() {
                write_png(&format!("{out}/strips_new/{name}.png"), &sheet(b, 5));
            }
        }
        if !sheet_old.is_empty() {
            write_png(&format!("{out}/sheet_old.png"), &sheet(&sheet_old, 6));
            let pairs: Vec<_> = sheet_old.iter().zip(&sheet_new).flat_map(|(a, b)| [a.clone(), b.clone()]).collect();
            write_png(&format!("{out}/sheet_pairs.png"), &sheet(&pairs, 4));
        }
        write_png(&format!("{out}/sheet_new.png"), &sheet(&sheet_new, 6));
        if video {
            encode(&frames_dir, &format!("{out}/scene_{view}.mp4"));
        }
        println!(
            "wrote {out}/ (sheet_new.png{})",
            if s.legacy.is_some() || tw.is_some() { ", sheet_pairs.png: each pair the old (or --against), then the new" } else { "" }
        );
    }
}

fn encode(frames: &str, out: &str) {
    let ok = std::process::Command::new("ffmpeg")
        .args([
            "-y",
            "-loglevel",
            "error",
            "-framerate",
            "30",
            "-i",
            &format!("{frames}/%04d.png"),
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
            "-crf",
            "20",
            out,
        ])
        .status()
        .is_ok_and(|s| s.success());
    println!("{}", if ok { format!("wrote {out}") } else { format!("ffmpeg failed: the frames are in {frames}") });
}

/// All six through their scenes at once, each in its own column: a row a second.
fn lineup_offscreen(args: &[String], dir: &str) {
    let libs = libraries(args);
    let mut r = renderer();
    let (w, h) = (300, 360);
    let mut all: Vec<(Stage, Specimen)> = KINDS
        .iter()
        .map(|k| {
            let st = Stage::new(w as f64 / h as f64);
            let s = specimen(k, &libs);
            st.scene.add(&s.rig.root);
            (st, s)
        })
        .collect();
    let fps = 30.0;
    let mut frames = vec![];
    for f in 0..(24.0 * fps) as usize {
        let mut row = vec![];
        for (st, s) in all.iter_mut() {
            let eye = s.eye();
            s.step(1.0 / fps, eye);
            if f % fps as usize == 0 {
                st.look(&camera("side", s.pos, eye, s.kind));
                row.push(st.render(&mut r, w, h));
            }
        }
        frames.extend(row);
    }
    write_png(&format!("{dir}/lineup.png"), &sheet(&frames, KINDS.len()));
    println!("wrote {dir}/lineup.png (columns: {}; a row a second)", KINDS.map(creatures::title).join(", "));
}

/// Each clip on its body in place: a sheet each, a frame every 0.25 s.
fn clips_offscreen(args: &[String], dir: &str) {
    let libs = libraries(args);
    let mut r = renderer();
    let (w, h) = (360, 420);
    for kind in chosen(args) {
        let mut s = specimen(kind, &libs);
        let Some(lib) = s.anim.as_ref().map(|a| a.lib) else { continue };
        let stage = Stage::new(w as f64 / h as f64);
        stage.scene.add(&s.rig.root);
        for (ci, c) in lib.clips.iter().enumerate() {
            // (its bridges too only when asked: there are hundreds)
            if c.bridge.is_some() && !flag(args, "--bridges") {
                continue;
            }
            s.clip = Some((ci, 0.0));
            if let Some(a) = s.anim.as_mut() {
                a.replant();
            }
            let mut frames = vec![];
            let len = c.duration(lib.fps).min(6.0);
            let n = (len * 30.0) as usize;
            for f in 0..n {
                s.step(1.0 / 30.0, v3(0.0, 1.6, 6.0));
                if f % 8 == 0 {
                    stage.look(&Orbit { target: [0.0, if kind == "hound" { 0.6 } else { 1.05 }, 0.0], yaw: 1.15, pitch: 0.1, dist: 4.0 });
                    frames.push(stage.render(&mut r, w, h));
                }
            }
            write_png(&format!("{dir}/{kind}/{}.png", c.name), &sheet(&frames, 8));
            println!(
                "{kind}.{:<12} {:>4} frames {:5.2} s  speed {:.2} m/s  “{}”",
                c.name,
                c.frames.len(),
                c.duration(lib.fps),
                c.speed,
                c.prompt
            );
        }
    }
}

/// The motion's numbers over each scene, the old animation and the new: how
/// far a planted foot (or hand) slides, how smooth the body moves, how much
/// of it moves at all, how still it is at rest.
fn metrics(args: &[String]) {
    let libs = libraries(args);
    let mut beats_out = String::new();
    let mut out = String::from(
        "| creature | foot slide, old | new | jerk m/s³, old | new | turning rad/s, old | new | at rest rad/s, old | new |\n|---|---|---|---|---|---|---|---|---|\n",
    );
    for kind in chosen(args) {
        if kind == "watcher" {
            continue;
        }
        let mut tw = twin(args, kind);
        let mut s = if tw.is_some() { specimen(kind, &libs) } else { specimen(kind, &libs).with_legacy() };
        let fps = 30.0;
        let mut names = vec![
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
        names.retain(|n| s.rig.ix(n).is_some());
        // what touches the floor: each foot's ankle and ball (the Hound's hands too)
        let mut contact: Vec<usize> =
            ["foot.L", "toes.L", "foot.R", "toes.R"].iter().filter_map(|n| names.iter().position(|m| m == n)).collect();
        if kind == "hound" {
            contact.extend(["hand.L", "hand.R"].iter().filter_map(|n| names.iter().position(|m| m == n)));
        }
        let world = |rig: &BodyRig, i: usize| {
            let p = rig.head(i);
            let (pos, yaw) = (rig.root.pos(), rig.root.rotation().y);
            let (sn, c) = yaw.dsin_cos();
            v3(pos.x + p.x * c + p.z * sn, p.y, pos.z - p.x * sn + p.z * c)
        };
        let mut tracks: [Vec<Vec<Vec3>>; 2] = [vec![], vec![]];
        let mut quats: [Vec<Vec<Quat>>; 2] = [vec![], vec![]];
        let mut rest: Vec<bool> = vec![];
        let mut beats: Vec<usize> = vec![];
        let len: f64 = s.scene.iter().map(|b| b.secs).sum();
        let mut last_bridge = None;
        for _ in 0..(len * fps) as usize {
            let dt = 1.0 / fps;
            let eye = s.eye();
            if s.step(dt, eye) {
                break;
            }
            s.pose_legacy(dt, eye, 0.0);
            if let Some(t) = tw.as_mut() {
                t.step(dt, eye);
            }
            trace(&s);
            if std::env::var("LAB_BRIDGES").is_ok()
                && let Some(a) = s.anim.as_ref()
            {
                let now = a.bridging();
                if now != last_bridge {
                    if let Some(b) = now {
                        println!("{kind} beat {:02} {:5.2} s: {b} (at {:.2} s)", s.beat, s.beat_t, s.time);
                    }
                    last_bridge = now;
                }
            }
            beats.push(s.beat);
            rest.push(
                matches!(s.mode, game::entities::brain::Mode::Idle | game::entities::brain::Mode::Hold) && s.vel.0.dhypot(s.vel.1) < 0.05,
            );
            let old = tw.as_ref().map(|t| &t.rig).unwrap_or_else(|| &s.legacy.as_ref().expect("the old animation").rig);
            for (k, rig) in [old, &s.rig].into_iter().enumerate() {
                tracks[k].push(names.iter().map(|nm| world(rig, rig.ix(nm).unwrap())).collect());
                quats[k].push(names.iter().map(|nm| rig.body_rot(rig.ix(nm).unwrap())).collect());
            }
        }
        let in_beat = |t: usize, b: Option<usize>| b.is_none_or(|b| beats[t] == b && (t == 0 || beats[t - 1] == b));
        let slide_in = |k: usize, b: Option<usize>| {
            let tr = &tracks[k];
            let (mut sum, mut cnt) = (0.0f64, 0.0f64);
            for &i in &contact {
                let floor = tr.iter().map(|f| f[i].y).fold(f64::MAX, f64::min);
                for t in 1..tr.len() {
                    if in_beat(t, b) && tr[t][i].y < floor + 0.015 && tr[t - 1][i].y < floor + 0.015 {
                        sum += (tr[t][i].x - tr[t - 1][i].x).dhypot(tr[t][i].z - tr[t - 1][i].z) * fps;
                        cnt += 1.0;
                    }
                }
            }
            sum / cnt.max(1.0)
        };
        let slide = |k: usize| slide_in(k, None);
        let jerk_in = |k: usize, b: Option<usize>| {
            let tr = &tracks[k];
            let d = |a: Vec3, b: Vec3| v3(a.x - b.x, a.y - b.y, a.z - b.z);
            let (mut sum, mut cnt) = (0.0, 0.0);
            for t in 3..tr.len() {
                if !in_beat(t, b) || !in_beat(t - 3, b) {
                    continue;
                }
                for i in 0..names.len() {
                    let j3 = d(
                        d(d(tr[t][i], tr[t - 1][i]), d(tr[t - 1][i], tr[t - 2][i])),
                        d(d(tr[t - 1][i], tr[t - 2][i]), d(tr[t - 2][i], tr[t - 3][i])),
                    );
                    sum += j3.length() * fps * fps * fps;
                    cnt += 1.0;
                }
            }
            sum / f64::max(cnt, 1.0)
        };
        let jerk = |k: usize| jerk_in(k, None);
        let turning = |k: usize, only_rest: bool| {
            let q = &quats[k];
            let (mut sum, mut cnt) = (0.0, 0.0);
            for t in 1..q.len() {
                if only_rest && !(rest[t] && rest[t - 1]) {
                    continue;
                }
                for i in 0..names.len() {
                    sum += 2.0 * q[t][i].dot(&q[t - 1][i]).abs().min(1.0).dacos() * fps;
                    cnt += 1.0;
                }
            }
            sum / f64::max(cnt, 1.0)
        };
        out += &format!(
            "| {} | {:.3} m/s | {:.3} m/s | {:.0} | {:.0} | {:.2} | {:.2} | {:.3} | {:.3} |\n",
            creatures::title(kind),
            slide(0),
            slide(1),
            jerk(0),
            jerk(1),
            turning(0, false),
            turning(1, false),
            turning(0, true),
            turning(1, true)
        );
        if std::env::var("LAB_REST").is_ok() {
            // which bones turn at rest, and how much (rad/s), the new
            let q = &quats[1];
            let mut by: Vec<(f64, &str)> = (0..names.len())
                .map(|i| {
                    let (mut sum, mut cnt) = (0.0, 0.0);
                    for t in 1..q.len() {
                        if rest[t] && rest[t - 1] {
                            sum += 2.0 * q[t][i].dot(&q[t - 1][i]).abs().min(1.0).dacos() * fps;
                            cnt += 1.0;
                        }
                    }
                    (sum / f64::max(cnt, 1.0), names[i])
                })
                .collect();
            by.sort_by(|a, b| b.0.total_cmp(&a.0));
            println!("{kind} at rest: {}", by.iter().take(8).map(|(v, n)| format!("{n} {v:.3}")).collect::<Vec<_>>().join(", "));
        }
        beats_out +=
            &format!("\n{}, beat by beat:\n\n| beat | slide old | new | jerk old | new |\n|---|---|---|---|---|\n", creatures::title(kind));
        for (bi, b) in s.scene.iter().enumerate() {
            beats_out += &format!(
                "| {bi:02} {} ({}) | {:.3} | {:.3} | {:.0} | {:.0} |\n",
                b.label,
                b.mode.name(),
                slide_in(0, Some(bi)),
                slide_in(1, Some(bi)),
                jerk_in(0, Some(bi)),
                jerk_in(1, Some(bi))
            );
        }
    }
    out += &beats_out;
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
    if let Some(dir) = arg(&args, "--lineup") {
        lineup_offscreen(&args, &dir);
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
    if std::env::var("LAB_POSE").is_ok() {
        pose_report(&args);
        return;
    }
    let libs = libraries(&args);
    engine::app::run(
        engine::app::AppConfig { title: "creature lab".into(), width: 1280.0, height: 800.0, fullscreen: false },
        window::Lab::new(libs, seed),
    );
}

/// `LAB_TRACE=kind:beat`: each frame of that beat, its clip, pace, stride and feet.
pub fn trace(s: &Specimen) {
    let Ok(want) = std::env::var("LAB_TRACE") else { return };
    let Some((k, b)) = want.split_once(':') else { return };
    if k != s.kind || b.parse::<usize>().ok() != Some(s.beat) {
        return;
    }
    let Some(a) = s.anim.as_ref() else { return };
    let (clip, t) = a.playing();
    let clip = a.bridging().unwrap_or(clip);
    let l = a.locks();
    let foot = |n: &str| {
        let i = s.rig.ix(n).unwrap();
        let p = s.rig.head(i);
        let (sn, c) = s.yaw.dsin_cos();
        v3(s.pos.x + p.x * c + p.z * sn, p.y, s.pos.z - p.x * sn + p.z * c)
    };
    let (fl, fr) = (foot("foot.L"), foot("foot.R"));
    println!(
        "{:5.2} {clip} {t:5.2} pace {:.2} stride {:.2} v {:.2} | L {} gap {:.2} ({:+.2} {:.2} {:+.2}) | R {} gap {:.2} ({:+.2} {:.2} {:+.2})",
        s.beat_t,
        a.pace,
        a.stride,
        s.vel.0.dhypot(s.vel.1),
        l[0].0 as u8,
        l[0].1,
        fl.x,
        fl.y,
        fl.z,
        l[1].0 as u8,
        l[1].1,
        fr.x,
        fr.y,
        fr.z
    );
}

/// `LAB_POSE=kind`: each bone that the creature's first second of its scene
/// turns furthest from its rest pose (the body's frame), degrees.
pub fn pose_report(args: &[String]) {
    let Ok(kind) = std::env::var("LAB_POSE") else { return };
    let libs = libraries(args);
    let k: &'static str = KINDS.iter().copied().find(|k| *k == kind).unwrap_or("corner");
    let mut s = specimen(k, &libs);
    for _ in 0..30 {
        let eye = s.eye();
        s.step(1.0 / 30.0, eye);
    }
    let rig = &s.rig;
    let mut by: Vec<(f64, String, Vec3)> = (0..rig.bones.len())
        .map(|i| {
            let q = rig.body_rot(i);
            let ang = 2.0 * q.w.abs().min(1.0).dacos();
            let h = rig.head(i);
            let r = rig.def.bones[i].head;
            (ang.to_degrees(), rig.def.bones[i].name.clone(), v3(h.x - r[0], h.y - r[1], h.z - r[2]))
        })
        .collect();
    by.sort_by(|a, b| b.0.total_cmp(&a.0));
    for (a, n, d) in by.iter().take(14) {
        println!("{n:<14} {a:6.1}°  moved {:+.3} {:+.3} {:+.3}", d.x, d.y, d.z);
    }
}
