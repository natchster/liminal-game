//! The lab's window: the Smiler's scene, its clips one by one, and the
//! plant gallery. The window's title says what is shown.
//!
//! | key | does |
//! |---|---|
//! | Tab | the scene → one clip → the plants |
//! | Space, `.` | pause; a frame at a time while paused |
//! | O | the game's animation beside the new one, on the same path |
//! | W | the wrongness laid on the mocap: all, half, none |
//! | L | a study light, or a dim room and one lamp |
//! | ← → | the previous / next clip, or plant seed |
//! | H | the plants' health: all three, tended, left a season, left for years |
//! | R | the scene from its start |
//! | drag, wheel, ↑ ↓ | turn round, come nearer |
//! | Esc | quit |
use crate::legacy::Legacy;
use crate::motion::Library;
use crate::plants::{self, Species};
use crate::smiler::{SCENE, Smiler};
use crate::stage::{Light, Orbit, Stage};
use engine::app::{AppCtx, AppHandler, Input};
use engine::prelude::*;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_metal::{MTLCommandBuffer, MTLTexture};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum View {
    Scene,
    Clip,
    Plants,
}

pub struct Lab {
    stage: Option<Stage>,
    lib: Option<Library>,
    smiler: Option<Smiler>,
    legacy: Option<Legacy>,
    view: View,
    paused: bool,
    step: bool,
    side_by_side: bool,
    orbit: Orbit,
    drag: Option<(f64, f64)>,
    cursor: (f64, f64),
    clip: usize,
    clip_t: f64,
    seed: u64,
    health: usize,
    plants: Option<Obj>,
    title: String,
    mode_t: f64,
    last_mode: Option<crate::smiler::Mode>,
}

impl Lab {
    pub fn new(lib: Option<Library>, seed: u64) -> Lab {
        Lab {
            stage: None,
            lib,
            smiler: None,
            legacy: None,
            view: View::Scene,
            paused: false,
            step: false,
            side_by_side: false,
            orbit: Orbit { target: [0.0, 1.0, 0.0], yaw: 0.6, pitch: 0.18, dist: 7.5 },
            drag: None,
            cursor: (0.0, 0.0),
            clip: 0,
            clip_t: 0.0,
            seed,
            health: 0,
            plants: None,
            title: String::new(),
            mode_t: 0.0,
            last_mode: None,
        }
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

    fn show(&mut self) {
        let plants = self.view == View::Plants;
        if let Some(p) = &self.plants {
            p.set_visible(plants);
        }
        if let Some(s) = &self.smiler {
            s.rig.root.set_visible(!plants);
        }
        if let Some(l) = &self.legacy {
            l.rig.root.set_visible(!plants && self.side_by_side);
        }
        self.orbit = match self.view {
            View::Scene => Orbit { target: [0.0, 1.0, 0.0], yaw: 0.6, pitch: 0.18, dist: 7.5 },
            View::Clip => Orbit { target: [0.0, 1.1, 0.0], yaw: 1.0, pitch: 0.12, dist: 4.2 },
            View::Plants => Orbit { target: [0.0, 0.5, 0.0], yaw: 0.4, pitch: 0.35, dist: 5.0 },
        };
    }

    fn set_title(&mut self, ctx: &AppCtx) {
        let what = match self.view {
            View::Plants => {
                let hs = ["tended, left a season, left for years", "tended", "left a season", "left for years"][self.health % 4];
                format!("plants (seed {}): {hs}", self.seed)
            }
            View::Clip => match &self.lib {
                Some(l) => format!(
                    "clip {}/{}: {}: \u{201c}{}\u{201d}",
                    self.clip + 1,
                    l.clips.len(),
                    l.clips[self.clip].name,
                    l.clips[self.clip].prompt
                ),
                None => "no clips".into(),
            },
            View::Scene => match &self.smiler {
                Some(s) => format!(
                    "the Smiler: {:?} ({}), wrongness {:.1}{}",
                    s.mode(),
                    SCENE[s.beat].clip,
                    s.wrong,
                    if self.side_by_side { "; left: the game's" } else { "" }
                ),
                None => "no clips: run lab/motion/motion.py bake".into(),
            },
        };
        let t = format!("creature lab: {what}{}", if self.paused { " (paused)" } else { "" });
        if t != self.title {
            ctx.window.set_title(&t);
            self.title = t;
        }
    }
}

impl AppHandler for Lab {
    fn init(&mut self, ctx: &mut AppCtx) {
        let stage = Stage::new(ctx.width / ctx.height);
        if let Some(lib) = &self.lib {
            self.smiler = Smiler::new(lib);
            self.legacy = Legacy::new();
        }
        if let Some(s) = &self.smiler {
            stage.scene.add(&s.rig.root);
        }
        if let Some(l) = &self.legacy {
            stage.scene.add(&l.rig.root);
        }
        self.stage = Some(stage);
        self.rebuild_plants();
        self.show();
    }

    fn frame(&mut self, ctx: &mut AppCtx, dt: f64) -> Option<Retained<ProtocolObject<dyn MTLCommandBuffer>>> {
        let dt = dt.min(0.05);
        let run = !self.paused || std::mem::take(&mut self.step);
        let dt = if self.paused { 1.0 / 30.0 } else { dt };
        let eye_o = self.orbit.eye();
        let eye = v3(eye_o[0], eye_o[1], eye_o[2]);
        if run && let (Some(lib), Some(s)) = (&self.lib, self.smiler.as_mut()) {
            match self.view {
                View::Scene => {
                    s.update(lib, dt, eye);
                    let mode = s.mode();
                    if Some(mode) != self.last_mode {
                        self.mode_t = 0.0;
                        self.last_mode = Some(mode);
                    }
                    self.mode_t += dt;
                    if self.side_by_side
                        && let Some(l) = self.legacy.as_mut()
                    {
                        // beside it, on the same path
                        let p = s.pos;
                        let off = v3(-1.4 * s.yaw.dcos(), 0.0, 1.4 * s.yaw.dsin());
                        let prev = l.rig.root.pos();
                        let at = v3(p.x + off.x, 0.0, p.z + off.z);
                        let (vx, vz) = ((at.x - prev.x) / dt, (at.z - prev.z) / dt);
                        l.animate(at, s.yaw, vx, vz, mode, self.mode_t, eye, s.time, dt);
                    }
                    self.orbit.target = [s.pos.x * 0.6, 1.0, s.pos.z * 0.6];
                }
                View::Clip => {
                    // one clip in place, looping
                    self.clip_t += dt;
                    let c = &lib.clips[self.clip];
                    let fr = c.sample(self.clip_t, lib.fps);
                    s.pos = v3(0, 0, 0);
                    s.yaw = 0.0;
                    s.rig.root.set_pos(0, 0, 0);
                    s.rig.root.rotation().y = 0.0;
                    s.rig.root.update_matrix_world(true);
                    s.rt.pose(&s.rig, &fr.p, &crate::motion::Gains::default(), [None, None]);
                    s.rig.sync();
                }
                View::Plants => {}
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
                "Tab" => {
                    self.view = match self.view {
                        View::Scene => View::Clip,
                        View::Clip => View::Plants,
                        View::Plants => View::Scene,
                    };
                    self.show();
                }
                "Space" => self.paused = !self.paused,
                "Period" => self.step = true,
                "KeyO" => {
                    self.side_by_side = !self.side_by_side;
                    if let Some(l) = self.legacy.as_mut() {
                        l.reset();
                    }
                    self.show();
                }
                "KeyW" => {
                    if let Some(s) = self.smiler.as_mut() {
                        s.wrong = if s.wrong > 0.75 {
                            0.5
                        } else if s.wrong > 0.25 {
                            0.0
                        } else {
                            1.0
                        };
                    }
                }
                "KeyL" => {
                    if let Some(st) = self.stage.as_mut() {
                        let l = if st.light == Light::Study { Light::Lamp } else { Light::Study };
                        st.set_light(l);
                    }
                }
                "KeyR" => {
                    if let (Some(lib), Some(s)) = (&self.lib, self.smiler.as_mut()) {
                        s.restart(lib);
                    }
                    if let Some(l) = self.legacy.as_mut() {
                        l.reset();
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
                            if let Some(lib) = &self.lib {
                                let n = lib.clips.len() as i64;
                                self.clip = ((self.clip as i64 + d).rem_euclid(n)) as usize;
                                self.clip_t = 0.0;
                            }
                        }
                    }
                }
                "ArrowUp" => self.orbit.dist = (self.orbit.dist * 0.9).max(1.0),
                "ArrowDown" => self.orbit.dist = (self.orbit.dist * 1.1).min(30.0),
                _ => {}
            },
            Input::MouseDown(_) => self.drag = Some(self.cursor),
            Input::MouseUp(_) => self.drag = None,
            Input::CursorPos(x, y) => {
                if let Some((px, py)) = self.drag {
                    self.orbit.yaw -= (x - px) * 0.008;
                    self.orbit.pitch = (self.orbit.pitch + (y - py) * 0.006).clamp(-0.2, 1.4);
                    self.drag = Some((x, y));
                }
                self.cursor = (x, y);
            }
            Input::Wheel(d) => self.orbit.dist = (self.orbit.dist * (1.0 + d.signum() * 0.08)).clamp(1.0, 30.0),
            _ => {}
        }
    }
}
