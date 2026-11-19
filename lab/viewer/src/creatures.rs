//! Every creature on the lab's stage, moved by the game's own code: its
//! body from `bodies.bin`, its clips (`entities::motion`), and its kind's
//! `pose` (`entities::smiler::pose`, `hound::pose`, ...), the same function
//! the game calls each frame. What the lab supplies is what the game's brain
//! and steering would: a mode, a place, a heading and a velocity ([`Beat`]s
//! of a scene, or the window's free driving), and the viewer's eye.
//!
//! The Watcher is never seen to move (design: it moves only unseen), so its
//! scene is the poses you would catch it in each time you looked back, a
//! little nearer each time.
use crate::legacy::{Inputs, Legacy};
use engine::prelude::*;
use game::entities::body::{BodyRig, build_body};
use game::entities::brain::Mode;
use game::entities::glass::AtGlass;
use game::entities::motion::anim::Anim;
use game::entities::motion::{Library, Mind, body_to_world};
use game::entities::reader::Page;

/// The kinds, in the order the window's number keys take them.
pub const KINDS: [&str; 6] = ["smiler", "hound", "watcher", "corner", "reader", "glass"];

pub fn title(kind: &str) -> &'static str {
    match kind {
        "smiler" => "the Smiler",
        "hound" => "the Hound",
        "watcher" => "the Watcher",
        "corner" => "the one in the corner",
        "reader" => "the Reader",
        "glass" => "the one behind the glass",
        _ => "?",
    }
}

pub fn material(kind: &str) -> fn(&str) -> Mat {
    match kind {
        "smiler" => game::entities::smiler::material,
        "hound" => game::entities::hound::material,
        "watcher" => game::entities::watcher::material,
        "corner" => game::entities::corner::material,
        "reader" => game::entities::reader::material,
        _ => game::entities::glass::material,
    }
}

/// Its hands are feet too.
fn fours(kind: &str) -> bool {
    kind == "hound"
}

/// The clip it starts on.
fn first(kind: &str) -> &'static str {
    match kind {
        "smiler" => "idle",
        "hound" => "crawl",
        "corner" => "hold",
        "reader" => "read_walk",
        _ => "still",
    }
}

/// The modes its brain puts it in (the window's free driving cycles them).
pub fn modes(kind: &str) -> &'static [Mode] {
    match kind {
        "smiler" => &[Mode::Idle, Mode::Alert, Mode::Search, Mode::Stalk, Mode::Hunt, Mode::Strike, Mode::Withdraw],
        "hound" => &[Mode::Roam, Mode::Alert, Mode::Search, Mode::Stalk, Mode::Hunt, Mode::Strike, Mode::Withdraw],
        "watcher" => &[Mode::Idle, Mode::Hold, Mode::Stalk],
        "corner" => &[Mode::Hold, Mode::Alert, Mode::Hunt, Mode::Strike, Mode::Withdraw],
        "reader" => &[Mode::Roam, Mode::Alert, Mode::Search, Mode::Hunt, Mode::Read, Mode::Strike, Mode::Withdraw],
        _ => &[Mode::Hold, Mode::Alert, Mode::Stalk, Mode::Hunt, Mode::Strike, Mode::Withdraw],
    }
}

/// How the creature's root goes during a beat (the game's steering, as the lab plays it).
#[derive(Clone, Copy, Debug)]
pub enum Root {
    /// where it is, turned as it is
    Still,
    /// where it is, turning to face the viewer
    Face,
    /// toward a point on the floor at a speed (m/s), turning toward it
    /// (`facing`: facing the viewer whatever way it goes, as the one behind
    /// the glass does)
    Steer { to: (f64, f64), speed: f64, facing: bool },
}

/// A stretch of a creature's scene.
#[derive(Clone, Copy, Debug)]
pub struct Beat {
    pub mode: Mode,
    pub secs: f64,
    pub root: Root,
    /// it has you: it stands over your body (your eye on the floor)
    pub killer: bool,
    pub page: Page,
    pub glass: AtGlass,
    /// the Watcher: which of its poses
    pub pose: usize,
    pub label: &'static str,
}

const fn beat(label: &'static str, mode: Mode, secs: f64, root: Root) -> Beat {
    Beat {
        mode,
        secs,
        root,
        killer: false,
        page: Page { stage: 0.0, t: 0.0, tears: false },
        glass: AtGlass { hands: None, pressing: false, pass: 0.0, inside: false, moving: false },
        pose: 0,
        label,
    }
}

const fn steer(x: f64, z: f64, speed: f64) -> Root {
    Root::Steer { to: (x, z), speed, facing: false }
}

/// Where the viewer stands in every scene.
pub const EYE: (f64, f64, f64) = (0.4, 1.6, 7.6);

/// Where each kind's scene starts (x, z, yaw).
pub fn start(kind: &str) -> (f64, f64, f64) {
    match kind {
        "smiler" => (-1.6, -3.4, -2.4),
        "hound" => (-4.5, -3.0, PI / 2.0),
        "watcher" => (0.0, -3.5, 0.0),
        "corner" => (-5.4, -5.4, -2.356),
        "reader" => (-4.5, -2.0, PI / 2.0),
        _ => (0.0, -4.5, 0.0),
    }
}

/// Each kind's scene: through its modes as its brain and the director
/// would take it toward a viewer (`docs/design/ENTITIES.md`, its Modes).
pub fn scene(kind: &str) -> Vec<Beat> {
    let over = |b: Beat| Beat { killer: true, ..b };
    match kind {
        "smiler" => vec![
            beat("in its dark, facing the corner", Mode::Idle, 5.0, Root::Still),
            beat("your light: its head first", Mode::Alert, 1.4, Root::Still),
            beat("then its body", Mode::Alert, 1.6, Root::Face),
            beat("to where your light went out", Mode::Search, 6.0, steer(0.2, -0.8, 1.6)),
            beat("there, it listens", Mode::Search, 2.6, Root::Still),
            beat("it sees you: it comes", Mode::Hunt, 2.0, steer(0.0, 2.6, 3.2)),
            beat("at 4.5 m/s", Mode::Hunt, 1.2, steer(0.1, 5.0, 4.5)),
            beat("you ran: it coils, it lunges", Mode::Strike, 1.0, steer(0.3, 6.8, 7.0)),
            over(beat("over your body", Mode::Strike, 3.0, Root::Still)),
            beat("back into the dark", Mode::Withdraw, 6.0, steer(-2.5, -3.0, 1.6)),
        ],
        "hound" => vec![
            beat("prowling, its head to the floor", Mode::Roam, 4.0, steer(-1.0, -3.0, 0.9)),
            beat("it stops dead to listen (the tell)", Mode::Alert, 2.2, Root::Still),
            beat("its lope along the hall", Mode::Roam, 2.5, steer(1.5, -3.0, 2.0)),
            beat("stalking toward a sound", Mode::Stalk, 3.0, steer(2.5, -1.5, 0.6)),
            beat("on, listening as it goes", Mode::Roam, 2.0, steer(2.5, -0.5, 2.0)),
            beat("a sound: it goes to it", Mode::Search, 2.0, steer(1.0, 2.0, 3.4)),
            beat("and sniffs about the spot", Mode::Search, 2.5, Root::Still),
            beat("it is sure: the bay, the charge", Mode::Hunt, 1.6, steer(0.5, 5.6, 6.0)),
            beat("it stops to listen", Mode::Hunt, 2.0, Root::Face),
            beat("a crouch, then it springs", Mode::Strike, 0.9, steer(0.4, 6.9, 7.0)),
            over(beat("over your body", Mode::Strike, 3.0, Root::Still)),
            beat("away", Mode::Withdraw, 5.0, steer(-4.0, -3.0, 2.0)),
        ],
        "watcher" => {
            let mut v = vec![beat("among the sheeted furniture", Mode::Idle, 3.0, Root::Face)];
            for (k, label) in ["you look back: it stands", "it listens", "bowed", "reaching", "leaning"].iter().enumerate() {
                let mut b = beat(label, Mode::Hold, 2.2, Root::Still);
                b.pose = k;
                v.push(b);
                // unseen, it came nearer (never while you look)
                v.push(beat(
                    "(unseen)",
                    Mode::Stalk,
                    0.5,
                    Root::Steer { to: (0.0, -2.5 + 1.6 * (k + 1) as f64), speed: 9.0, facing: true },
                ));
            }
            v.push(over(Beat { pose: 5, ..beat("bent over you", Mode::Strike, 3.0, Root::Still) }));
            v
        }
        "corner" => vec![
            beat("in its corner, forehead to the angle of the walls", Mode::Hold, 5.0, Root::Still),
            beat("its face comes round (the hum stops)", Mode::Alert, 2.4, Root::Still),
            beat("you held the look: it screams and comes", Mode::Hunt, 2.4, steer(0.0, 5.6, 5.6)),
            beat("it reaches you", Mode::Strike, 0.6, steer(0.3, 6.8, 5.6)),
            over(beat("over your body", Mode::Strike, 3.0, Root::Still)),
            beat("back to its corner", Mode::Withdraw, 12.0, steer(-5.4, -5.4, 1.2)),
            beat("and stands as it was told to", Mode::Hold, 3.0, Root::Still),
        ],
        "reader" => {
            let read =
                |stage: f64, label, secs, tears| Beat { page: Page { stage, t: 0.0, tears }, ..beat(label, Mode::Read, secs, Root::Still) };
            vec![
                beat("reading as it walks", Mode::Roam, 6.5, steer(1.5, -2.0, 0.9)),
                beat("it looks up over its page", Mode::Alert, 1.6, Root::Face),
                beat("it snaps the sheet flat and comes", Mode::Hunt, 1.6, steer(0.4, 2.6, 4.8)),
                Beat {
                    page: Page { stage: 0.0, t: 0.0, tears: false },
                    ..beat("a sheet on the floor: it must", Mode::Read, 0.6, steer(0.4, 3.4, 2.2))
                },
                read(1.0, "it stoops for it", 0.8, false),
                read(2.0, "and reads it to the end", 3.0, false),
                beat("it looks up", Mode::Alert, 1.0, Root::Face),
                beat("and comes", Mode::Hunt, 0.7, steer(0.4, 5.6, 4.8)),
                read(1.0, "another: it stoops", 0.8, true),
                read(2.0, "and tears it in half unread (learned)", 1.0, true),
                beat("the sheet lowered as it comes", Mode::Strike, 0.8, steer(0.4, 6.8, 4.8)),
                over(beat("over your body", Mode::Strike, 3.0, Root::Still)),
                beat("away, reading", Mode::Withdraw, 6.0, steer(-4.0, -2.0, 0.9)),
            ]
        }
        _ => {
            let g = |label, mode, secs, root, glass: AtGlass| Beat { glass, ..beat(label, mode, secs, root) };
            let walk = AtGlass { moving: true, ..AtGlass::default() };
            // a pane 0.33 m before it, its hands either side of its face
            let hands = Some([v3(0.22, 1.35, 0.33), v3(-0.22, 1.35, 0.33)]);
            let press = AtGlass { hands, pressing: true, ..AtGlass::default() };
            let inside = AtGlass { inside: true, moving: true, ..AtGlass::default() };
            let mut v = vec![
                g("far out on the track, facing you", Mode::Hold, 3.0, Root::Face, AtGlass::default()),
                g("your eyes on its glass: it comes", Mode::Stalk, 2.0, Root::Steer { to: (0.0, -0.5), speed: 2.6, facing: true }, walk),
                g("you looked away: it stands", Mode::Alert, 1.6, Root::Face, AtGlass::default()),
                g("at the glass: its face to the pane", Mode::Hunt, 3.0, Root::Still, press),
            ];
            for k in 0..6 {
                let pass = (k as f64 + 0.5) / 6.0;
                v.push(g(
                    "through it, bent low",
                    Mode::Hunt,
                    0.4,
                    Root::Steer { to: (0.0, 0.6), speed: 0.46, facing: true },
                    AtGlass { pass, ..AtGlass::default() },
                ));
            }
            v.extend([
                g("inside, its arms out to you", Mode::Hunt, 3.0, Root::Steer { to: (0.2, 5.4), speed: 1.8, facing: true }, inside),
                g("it takes you", Mode::Strike, 0.8, Root::Steer { to: (0.3, 6.8), speed: 1.8, facing: true }, inside),
                over(g("over your body", Mode::Strike, 3.0, Root::Still, AtGlass { inside: true, ..AtGlass::default() })),
                g("back out, facing you", Mode::Withdraw, 4.0, Root::Steer { to: (0.0, 1.0), speed: 1.8, facing: true }, walk),
            ]);
            v
        }
    }
}

/// The window's free driving: a mode, a speed round the stage, over you or not.
#[derive(Clone, Copy, Debug)]
pub struct Free {
    pub mode: usize,
    pub speed: f64,
    pub killer: bool,
    /// round the circle, radians
    pub at: f64,
}

/// A creature on the stage.
pub struct Specimen {
    pub kind: &'static str,
    pub rig: BodyRig,
    pub anim: Option<Anim>,
    pub legacy: Option<Legacy>,
    pub pos: Vec3,
    pub yaw: f64,
    pub beat: usize,
    pub beat_t: f64,
    pub scene: Vec<Beat>,
    pub time: f64,
    /// the mode it is in, and for how long
    pub mode: Mode,
    pub mode_t: f64,
    pub chase_t: f64,
    /// the one in the corner's head (its rule's: `corner::head_turn`)
    head_yaw: f64,
    /// where in its scene, along x (the line-up puts them side by side)
    pub offset: f64,
    pub free: Option<Free>,
    /// the velocity it moved at last frame
    pub vel: (f64, f64),
    /// a clip shown in place (the window's clip view): which, and when
    pub clip: Option<(usize, f64)>,
}

fn wrap(a: f64) -> f64 {
    (a + PI).rem_euclid(2.0 * PI) - PI
}

impl Specimen {
    /// Kind `kind`, its clips from `lib` (the game's if `None`).
    pub fn new(kind: &'static str, lib: Option<&'static Library>) -> Option<Specimen> {
        let rig = build_body(kind, &material(kind))?;
        let anim = match lib.or_else(|| game::entities::motion::library(kind)) {
            Some(lib) if kind != "watcher" => Some(Anim::new(&rig, lib, fours(kind), first(kind), 0x1ab ^ kind.len() as u64)?),
            _ => None,
        };
        let (x, z, yaw) = start(kind);
        let mut s = Specimen {
            kind,
            rig,
            anim,
            legacy: None,
            pos: v3(x, 0.0, z),
            yaw,
            beat: 0,
            beat_t: 0.0,
            scene: scene(kind),
            time: 0.0,
            mode: Mode::Idle,
            mode_t: 0.0,
            chase_t: 0.0,
            head_yaw: 0.0,
            offset: 0.0,
            free: None,
            vel: (0.0, 0.0),
            clip: None,
        };
        s.mode = s.scene[0].mode;
        s.place();
        Some(s)
    }

    /// The old animation beside it (the game's before clips), on the same path.
    pub fn with_legacy(mut self) -> Specimen {
        if self.kind != "watcher" {
            self.legacy = Legacy::new(self.kind);
        }
        self
    }

    pub fn current(&self) -> Beat {
        self.scene[self.beat]
    }

    /// Back to its scene's start.
    pub fn restart(&mut self) {
        let (x, z, yaw) = start(self.kind);
        self.pos = v3(x + self.offset, 0.0, z);
        self.yaw = yaw;
        self.beat = 0;
        self.beat_t = 0.0;
        self.time = 0.0;
        self.mode = self.scene[0].mode;
        self.mode_t = 0.0;
        self.chase_t = 0.0;
        self.head_yaw = 0.0;
        if let Some(a) = self.anim.as_mut() {
            a.replant();
        }
        if let Some(l) = self.legacy.as_mut() {
            l.reset();
        }
        self.place();
    }

    fn place(&self) {
        self.rig.root.set_pos(self.pos.x, self.pos.y, self.pos.z);
        self.rig.root.rotation().y = self.yaw;
        self.rig.root.update_matrix_world(true);
    }

    /// The viewer's eye: standing, or on the floor once it has you.
    pub fn eye(&self) -> Vec3 {
        let down = self.free.map_or(self.current().killer, |f| f.killer);
        v3(EYE.0 + self.offset, if down { 0.35 } else { EYE.1 }, EYE.2)
    }

    /// One step of `dt` seconds of its scene (or its free driving), `eye`
    /// what it watches; → whether its scene came round to the start.
    pub fn step(&mut self, dt: f64, eye: Vec3) -> bool {
        self.time += dt;
        if self.clip.is_some() {
            // a clip in place, facing the viewer's side of the stage
            self.pos = v3(self.offset, 0.0, 0.0);
            self.yaw = 0.0;
            self.vel = (0.0, 0.0);
            self.place();
            let b = beat("clip", Mode::Idle, 1e9, Root::Still);
            self.pose(&b, dt, eye);
            return false;
        }
        let mut wrapped = false;
        let b = match self.free {
            Some(f) => {
                let mode = modes(self.kind)[f.mode % modes(self.kind).len()];
                Beat { killer: f.killer, ..beat("free", mode, 1e9, Root::Still) }
            }
            None => {
                self.beat_t += dt;
                if self.beat_t >= self.current().secs {
                    self.beat_t = 0.0;
                    self.beat += 1;
                    if self.beat == self.scene.len() {
                        self.restart();
                        wrapped = true;
                    }
                }
                self.current()
            }
        };
        if b.mode != self.mode {
            self.mode = b.mode;
            self.mode_t = 0.0;
        }
        self.mode_t += dt;
        self.chase_t = if b.mode == Mode::Hunt { self.chase_t + dt } else { 0.0 };
        // ---- the root, as the brain and steering would move it
        let before = self.pos;
        let face = |s: &Specimen| (eye.x - s.pos.x).datan2(eye.z - s.pos.z);
        if let Some(f) = self.free.as_mut() {
            // round a circle of 2.5 m about the stage's middle, at its speed
            let r = 2.5;
            f.at += f.speed * dt / r;
            let (s, c) = f.at.dsin_cos();
            self.pos = v3(self.offset + r * s, 0.0, r * c);
            let along = f.at + PI / 2.0;
            let want = if f.speed > 0.05 { along } else { self.yaw };
            self.yaw += wrap(want - self.yaw).clamp(-3.0 * dt, 3.0 * dt);
        } else {
            match b.root {
                Root::Still => {}
                Root::Face => {
                    let want = face(self);
                    self.yaw += wrap(want - self.yaw).clamp(-2.4 * dt, 2.4 * dt);
                }
                Root::Steer { to, speed, facing } => {
                    let to = (to.0 + self.offset, to.1);
                    let (dx, dz) = (to.0 - self.pos.x, to.1 - self.pos.z);
                    let dist = dx.dhypot(dz);
                    let want = if facing { face(self) } else { dx.datan2(dz) };
                    self.yaw += wrap(want - self.yaw).clamp(-4.0 * dt, 4.0 * dt);
                    let step = (speed * dt).min(dist);
                    if dist > 1e-6 {
                        self.pos = v3(self.pos.x + dx / dist * step, 0.0, self.pos.z + dz / dist * step);
                    }
                    // there: the beat is over (a creature never walks on the spot)
                    if dist - step < 0.05 && self.beat_t > 0.1 && b.secs < 1e8 {
                        self.beat_t = b.secs;
                    }
                }
            }
        }
        self.vel = ((self.pos.x - before.x) / dt.max(1e-6), (self.pos.z - before.z) / dt.max(1e-6));
        // (the killer turned to you, as the game turns it)
        if b.killer && matches!(self.kind, "corner" | "reader") {
            self.yaw = face(self);
        }
        self.place();
        self.pose(&b, dt, eye);
        wrapped
    }

    fn mind(&self, b: &Beat, eye: Vec3) -> Mind {
        // (what it heard: about where you stand, as a sound tells it)
        let heard = Some(v3(eye.x, game::entities::motion::HEARD_Y, eye.z));
        Mind { mode: b.mode, mode_t: self.mode_t, killer: b.killer, eye, heard, chase_t: self.chase_t }
    }

    /// The game's own pose of it.
    fn pose(&mut self, b: &Beat, dt: f64, eye: Vec3) {
        let m = self.mind(b, eye);
        let rig = &self.rig;
        if self.kind == "watcher" {
            watcher_pose(rig, b.pose);
            rig.sync();
            return;
        }
        let Some(a) = self.anim.as_mut() else { return };
        let d = a.drive(self.pos, self.yaw, dt);
        if let Some((clip, t)) = self.clip.as_mut() {
            // one clip in place, bare: no wrongness laid on
            *t += dt;
            let lib: &'static Library = a.lib;
            let c = &lib.clips[*clip];
            let p = game::entities::motion::anim::Pick::new(c.name.as_str()).at(*t % c.duration(lib.fps)).fade(0.05);
            a.play(rig, &p, &d);
            rig.sync();
            return;
        }
        match self.kind {
            "smiler" => game::entities::smiler::pose(a, rig, &d, &m),
            "hound" => game::entities::hound::pose(a, rig, &d, &m),
            "corner" => {
                let want = wrap((eye.x - self.pos.x).datan2(eye.z - self.pos.z) - self.yaw);
                self.head_yaw = game::entities::corner::head_turn(m.mode, m.mode_t, m.killer, want, self.head_yaw, dt);
                game::entities::corner::pose(a, rig, &d, &m, self.head_yaw)
            }
            "reader" => {
                // (its rule's clock starts again at each stage, as `reader::read` does)
                let page = Page { t: self.beat_t, ..b.page };
                game::entities::reader::pose(a, rig, &d, &m, &page)
            }
            _ => game::entities::glass::pose(a, rig, &d, &m, &b.glass),
        }
        rig.sync();
    }

    /// The old animation, `beside` m to its left on the same path.
    pub fn pose_legacy(&mut self, dt: f64, eye: Vec3, beside: f64) {
        let b = match self.free {
            Some(f) => Beat { killer: f.killer, ..beat("free", modes(self.kind)[f.mode % modes(self.kind).len()], 1e9, Root::Still) },
            None => self.current(),
        };
        let at = body_to_world(self.pos, self.yaw, v3(beside, 0.0, 0.0));
        let page = Page { t: self.beat_t, ..b.page };
        let i = Inputs {
            pos: at,
            yaw: self.yaw,
            vel: self.vel,
            mode: b.mode,
            mode_t: self.mode_t,
            pp: eye,
            killer: b.killer,
            t: self.time,
            dt,
            chase_t: self.chase_t,
            page,
            glass: b.glass,
        };
        if let Some(l) = self.legacy.as_mut() {
            l.animate(&i);
        }
    }
}

/// The Watcher as it stands in pose `k` of `watcher::POSES` (the morphs of
/// its sheet: the first pose is its rest).
pub fn watcher_pose(rig: &BodyRig, k: usize) {
    let Some(sheet) = rig.meshes.get("sheet") else { return };
    let mut inf = sheet.morph_influences();
    let mut want = vec![0.0; inf.len()];
    if k > 0 && k <= want.len() {
        want[k - 1] = 1.0;
    }
    *inf = want;
}
