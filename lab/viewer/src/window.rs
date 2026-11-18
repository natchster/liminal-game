//! The lab's window: every creature, moved by the game's own code, through
//! its scene, driven by hand, or one clip at a time; and the plant gallery.
//! The window's title says what is shown and what it is doing.
//!
//! | key | does |
//! |---|---|
//! | 1–6 | the Smiler, the Hound, the Watcher, the one in the corner, the Reader, the one behind the glass |
//! | 0 | all six side by side |
//! | Tab | its scene → driven by hand → its clips one by one → the plants |
//! | [ ] | (by hand) the mode it is in |
//! | - = | (by hand) how fast it goes round: still, 0.5 … 6 m/s |
//! | K | (by hand) over your body, or not |
//! | ← → | the previous / next clip, or plant seed |
//! | O | the game's animation from before its clips, beside it on the same path |
//! | W | its wrongness laid on the mocap: all, half, none |
//! | L | a study light, or a dim room and one lamp |
//! | Space, `.` | pause; a frame at a time while paused |
//! | R | its scene from the start |
//! | H | the plants' health: all three, tended, left a season, left for years |
//! | drag, wheel, ↑ ↓ | turn round, come nearer |
//! | Esc | quit |
use crate::creatures::{Free, KINDS, Specimen, modes, title};
use crate::plants::{self, Species};
use crate::stage::{Light, Orbit, Stage};
use engine::app::{AppCtx, AppHandler, Input};
use engine::prelude::*;
use game::entities::motion::Library;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_metal::{MTLCommandBuffer, MTLTexture};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum View {
    Scene,
    Free,
    Clip,
    Plants,
}

/// How fast it goes round when driven by hand, m/s.
const SPEEDS: [f64; 9] = [0.0, 0.5, 1.0, 1.6, 2.2, 3.2, 4.5, 6.0, 7.0];

/// Side by side: x apart, m.
const APART: f64 = 2.4;

pub struct Lab {
    stage: Option<Stage>,
    libs: Vec<Option<&'static Library>>,
    specimens: Vec<Specimen>,
    /// which: 0..6, or 6 for all of them
    sel: usize,
    view: View,
    paused: bool,
    step: bool,
    side_by_side: bool,
    orbit: Orbit,
    follow: bool,
    drag: Option<(f64, f64)>,
    cursor: (f64, f64),
    clip: usize,
    speed: usize,
    seed: u64,
    health: usize,
    plants: Option<Obj>,
    title: String,
}

impl Lab {
    pub fn new(libs: Vec<Option<&'static Library>>, seed: u64) -> Lab {
        Lab {
            stage: None,
            libs,
            specimens: vec![],
            sel: 0,
            view: View::Scene,
            paused: false,
            step: false,
            side_by_side: false,
            orbit: Orbit { target: [0.0, 1.0, 0.0], yaw: 0.6, pitch: 0.18, dist: 7.5 },
            follow: true,
            drag: None,
            cursor: (0.0, 0.0),
            clip: 0,
            speed: 2,
            seed,
            health: 0,
            plants: None,
            title: String::new(),
        }
    }

    fn all(&self) -> bool {
        self.sel >= KINDS.len()
    }

    /// The specimens shown now.
    fn shown(&self) -> Vec<usize> {
        if self.all() { (0..self.specimens.len()).collect() } else { vec![self.sel] }
    }

    fn rebuild_plants(&mut self) {
        let Some(stage) = &self.stage else { return };
        if let Some(p) = self.plants.take() {
            stage.scene.remove(&p);
        }
        let g = group();
        let sets: [&[f64]; 4] = [&[1.0, 0.45, 0.0], &[1.0], &[0.45], &[0.0]];
        let hs = sets[self.health % 4];
        for (si, s) in Species::ALL.iter().enumerate() {
            for (hi, &h) in hs.iter().enumerate() {
                let node = plants::to_scene(&s.grow(self.seed + si as u64, h));
                node.set_pos((si as f64 - 1.0) * 1.6, 0.0, (hi as f64 - (hs.len() as f64 - 1.0) / 2.0) * -1.6);
                g.add(&node);
            }
        }
        g.set_visible(self.view == View::Plants);
        stage.scene.add(&g);
        self.plants = Some(g);
    }

    /// Put each creature where the view wants it, and show what is shown.
    fn show(&mut self) {
        let plants = self.view == View::Plants;
        if let Some(p) = &self.plants {
            p.set_visible(plants);
        }
        let all = self.all();
        let shown = self.shown();
        let n = KINDS.len() as f64;
        for (i, s) in self.specimens.iter_mut().enumerate() {
            s.offset = if all { (i as f64 - (n - 1.0) / 2.0) * APART } else { 0.0 };
            s.free = (self.view == View::Free).then_some(Free { mode: 0, speed: SPEEDS[self.speed], killer: false, at: 0.0 });
            s.clip = None;
            s.restart();
            let on = !plants && shown.contains(&i);
            s.rig.root.set_visible(on);
            if let Some(l) = &s.legacy {
                l.rig.root.set_visible(on && self.side_by_side && !all);
            }
        }
        if self.view == View::Clip {
            self.clip = 0;
            self.set_clip();
        }
        self.follow = true;
        self.orbit = match self.view {
            View::Plants => Orbit { target: [0.0, 0.5, 0.0], yaw: 0.4, pitch: 0.35, dist: 5.0 },
            View::Clip => Orbit { target: [0.0, 1.0, 0.0], yaw: 1.0, pitch: 0.12, dist: if all { 12.0 } else { 4.2 } },
            View::Free => Orbit { target: [0.0, 0.9, 0.0], yaw: 0.6, pitch: 0.35, dist: if all { 16.0 } else { 8.5 } },
            View::Scene => Orbit { target: [0.0, 1.0, 0.0], yaw: 0.6, pitch: 0.18, dist: if all { 16.0 } else { 7.5 } },
        };
    }

    fn set_clip(&mut self) {
        let k = self.clip;
        for s in self.specimens.iter_mut() {
            let n = s.anim.as_ref().map_or(0, |a| a.lib.clips.len());
            s.clip = (n > 0).then(|| (k % n, 0.0));
            if let Some(a) = s.anim.as_mut() {
                a.replant();
            }
        }
    }

    fn set_title(&mut self, ctx: &AppCtx) {
        let what = match self.view {
            View::Plants => {
                let hs = ["tended, left a season, left for years", "tended", "left a season", "left for years"][self.health % 4];
                format!("plants (seed {}): {hs}", self.seed)
            }
            _ if self.all() => match self.view {
                View::Free => format!("all six, by hand at {} m/s ([ ] mode, - = speed, K over you)", SPEEDS[self.speed]),
                View::Clip => format!("all six, clip {} each (← →)", self.clip + 1),
                _ => "all six, through their scenes (1–6 for one)".into(),
            },
            _ => {
                let Some(s) = self.specimens.get(self.sel) else { return };
                let who = title(s.kind);
                let playing = s.anim.as_ref().map_or_else(
                    || "its poses".to_string(),
                    |a| {
                        let (clip, t) = a.playing();
                        let held = if a.held() { ", held" } else { "" };
                        format!("clip {clip} {t:.1} s, pace ×{:.2} stride ×{:.2}{held}, wrongness {:.1}", a.pace.abs(), a.stride, a.wrong)
                    },
                );
                let old = if self.side_by_side && s.legacy.is_some() { "; beside it: the old animation" } else { "" };
                match self.view {
                    View::Clip => {
                        let c = s.anim.as_ref().zip(s.clip).map(|(a, (i, _))| {
                            let c = &a.lib.clips[i];
                            format!("clip {}/{}: {}: \u{201c}{}\u{201d}", i + 1, a.lib.clips.len(), c.name, c.prompt)
                        });
                        format!("{who}, {}", c.unwrap_or_else(|| "no clips (the Watcher has its poses)".into()))
                    }
                    View::Free => {
                        let f = s.free.unwrap_or(Free { mode: 0, speed: 0.0, killer: false, at: 0.0 });
                        format!(
                            "{who} by hand: {} at {} m/s{} ({playing}){old}",
                            s.mode.name(),
                            f.speed,
                            if f.killer { ", over your body" } else { "" }
                        )
                    }
                    _ => format!("{who}: {} ({}; {playing}){old}", s.current().label, s.mode.name()),
                }
            }
        };
        let t = format!("creature lab: {what}{}", if self.paused { " (paused)" } else { "" });
        if t != self.title {
            ctx.window.set_title(&t);
            self.title = t;
        }
    }

    fn free(&mut self, f: impl Fn(&mut Free)) {
        for s in self.specimens.iter_mut() {
            if let Some(fr) = s.free.as_mut() {
                f(fr);
            }
        }
    }
}

impl AppHandler for Lab {
    fn init(&mut self, ctx: &mut AppCtx) {
        let stage = Stage::new(ctx.width / ctx.height);
        for (i, k) in KINDS.iter().enumerate() {
            match Specimen::new(k, self.libs[i]) {
                Some(s) => {
                    let s = s.with_legacy();
                    stage.scene.add(&s.rig.root);
                    if let Some(l) = &s.legacy {
                        stage.scene.add(&l.rig.root);
                    }
                    self.specimens.push(s);
                }
                None => eprintln!("{k}: no body"),
            }
        }
        self.stage = Some(stage);
        self.rebuild_plants();
        self.show();
    }

    fn frame(&mut self, ctx: &mut AppCtx, dt: f64) -> Option<Retained<ProtocolObject<dyn MTLCommandBuffer>>> {
        let dt = dt.min(0.05);
        let run = !self.paused || std::mem::take(&mut self.step);
        let dt = if self.paused { 1.0 / 30.0 } else { dt };
        if run && self.view != View::Plants {
            let side = self.side_by_side && !self.all();
            for i in self.shown() {
                let s = &mut self.specimens[i];
                // in its scene it watches where you stand; driven by hand, the camera
                let eye = if self.view == View::Free {
                    let e = self.orbit.eye();
                    v3(e[0], e[1], e[2])
                } else {
                    s.eye()
                };
                s.step(dt, eye);
                if side {
                    s.pose_legacy(dt, eye, -1.4);
                }
            }
            if self.follow && !self.all() && self.view == View::Scene {
                let s = &self.specimens[self.sel];
                self.orbit.target = [s.pos.x * 0.6, if s.kind == "hound" { 0.7 } else { 1.0 }, s.pos.z * 0.6];
            }
        }
        self.set_title(ctx);
        let stage = self.stage.as_ref()?;
        stage.look(&self.orbit);
        let target = ctx.drawable?;
        let (w, h) = (target.width(), target.height());
        if let engine::scene::Kind::Camera(c) = &mut stage.camera.n().kind {
            c.aspect = w as f64 / h as f64;
        }
        Some(ctx.renderer.render(&stage.scene, &stage.camera, &stage.globals(), &[], target, w, h))
    }

    fn input(&mut self, ctx: &mut AppCtx, ev: Input) {
        match ev {
            Input::KeyDown(k, _) => match k.as_str() {
                "Escape" => ctx.quit(),
                "Digit0" => {
                    self.sel = KINDS.len();
                    self.show();
                }
                d if d.starts_with("Digit") => {
                    if let Some(n) = d[5..].parse::<usize>().ok().filter(|n| (1..=KINDS.len()).contains(n)) {
                        self.sel = n - 1;
                        self.show();
                    }
                }
                "Tab" => {
                    self.view = match self.view {
                        View::Scene => View::Free,
                        View::Free => View::Clip,
                        View::Clip => View::Plants,
                        View::Plants => View::Scene,
                    };
                    self.show();
                }
                "BracketLeft" | "BracketRight" => {
                    let d: i64 = if k == "BracketLeft" { -1 } else { 1 };
                    for s in self.specimens.iter_mut() {
                        let n = modes(s.kind).len() as i64;
                        if let Some(f) = s.free.as_mut() {
                            f.mode = ((f.mode as i64 + d).rem_euclid(n)) as usize;
                        }
                    }
                }
                "Minus" | "Equal" => {
                    self.speed = if k == "Minus" { self.speed.saturating_sub(1) } else { (self.speed + 1).min(SPEEDS.len() - 1) };
                    let v = SPEEDS[self.speed];
                    self.free(|f| f.speed = v);
                }
                "KeyK" => self.free(|f| f.killer = !f.killer),
                "Space" => self.paused = !self.paused,
                "Period" => self.step = true,
                "KeyO" => {
                    self.side_by_side = !self.side_by_side;
                    self.show();
                }
                "KeyW" => {
                    for s in self.specimens.iter_mut() {
                        if let Some(a) = s.anim.as_mut() {
                            a.wrong = if a.wrong > 0.75 {
                                0.5
                            } else if a.wrong > 0.25 {
                                0.0
                            } else {
                                1.0
                            };
                        }
                    }
                }
                "KeyL" => {
                    if let Some(st) = self.stage.as_mut() {
                        let l = if st.light == Light::Study { Light::Lamp } else { Light::Study };
                        st.set_light(l);
                    }
                }
                "KeyR" => {
                    for s in self.specimens.iter_mut() {
                        s.restart();
                    }
                }
                "KeyH" => {
                    self.health += 1;
                    self.rebuild_plants();
                }
                "ArrowLeft" | "ArrowRight" => {
                    let d: i64 = if k == "ArrowLeft" { -1 } else { 1 };
                    match self.view {
                        View::Plants => {
                            self.seed = (self.seed as i64 + d).max(0) as u64;
                            self.rebuild_plants();
                        }
                        _ => {
                            let n = self.specimens.get(self.sel).and_then(|s| s.anim.as_ref()).map_or(12, |a| a.lib.clips.len()) as i64;
                            self.clip = ((self.clip as i64 + d).rem_euclid(n.max(1))) as usize;
                            if self.view == View::Clip {
                                self.set_clip();
                            }
                        }
                    }
                }
                "ArrowUp" => self.orbit.dist = (self.orbit.dist * 0.9).max(1.0),
                "ArrowDown" => self.orbit.dist = (self.orbit.dist * 1.1).min(40.0),
                _ => {}
            },
            Input::MouseDown(_) => self.drag = Some(self.cursor),
            Input::MouseUp(_) => self.drag = None,
            Input::CursorPos(x, y) => {
                if let Some((px, py)) = self.drag {
                    self.orbit.yaw -= (x - px) * 0.008;
                    self.orbit.pitch = (self.orbit.pitch + (y - py) * 0.006).clamp(-0.2, 1.4);
                    self.drag = Some((x, y));
                    self.follow = false;
                }
                self.cursor = (x, y);
            }
            Input::Wheel(d) => self.orbit.dist = (self.orbit.dist * (1.0 + d.signum() * 0.08)).clamp(1.0, 40.0),
            _ => {}
        }
    }
}
