//! A cutscene as data: what `cargo xtask cutscene write` compiles from its
//! TOML source (`assets/src/cutscenes/ID.toml`) into the game's bundle
//! (`assets/cutscenes.json`), every field written out. The parse is strict:
//! a key it does not know, a missing one or a value of the wrong kind is an
//! error naming the cutscene and the field, so a source and the game can
//! never quietly disagree. Angles are in degrees (as written), times in
//! seconds of the cutscene's own clock.
use super::curve::Ease;
use serde_json::{Map, Value};

/// A jolt segment at most this long (s); a cutscene that gives your hands
/// back at most this much of them in all
pub const JOLT_MAX: f64 = 0.7;
pub const JOLT_BACK: f64 = 1.0;

/// Whether the world goes on while a cutscene plays
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum World {
    /// everything runs as in play (the creatures, the director, the place);
    /// only your body is still
    Live,
    /// only the place's light and sound run (the title's view of it)
    Held,
}

/// Where the camera (and your hands) go at the end
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Then {
    /// back in place: the last frame is the gameplay camera
    Back,
    /// to the death card (the camera stays where the cutscene left it)
    Card,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Skip {
    /// held for `play::SKIP_HOLD`, it lands where watching to the end would
    Hold,
    Never,
}

/// A frame a cutscene's keys are written in: x right, y up, z out of its
/// face (so -z is forward), level (its yaw only).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Anchor {
    /// the camera as it was when the cutscene began
    Start,
    /// your eyes, facing as you face (the gameplay camera without its sway)
    Eyes,
    /// the object it was started from, its front as +z
    Thing,
    /// what killed you: its head, live (followed as it moves)
    Killer,
}

impl Anchor {
    pub fn parse(s: &str) -> Option<Anchor> {
        Some(match s {
            "start" => Anchor::Start,
            "eyes" => Anchor::Eyes,
            "thing" => Anchor::Thing,
            "killer" => Anchor::Killer,
            _ => return None,
        })
    }
    pub fn name(self) -> &'static str {
        match self {
            Anchor::Start => "start",
            Anchor::Eyes => "eyes",
            Anchor::Thing => "thing",
            Anchor::Killer => "killer",
        }
    }
}

/// Where a key looks
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Look {
    /// the anchor's own facing, turned by the key's `yaw` and `pitch`
    Ahead,
    /// a point in the key's anchor frame
    Point([f64; 3]),
    /// an anchor's origin, live (`killer`: its head)
    At(Anchor),
}

/// A camera key
#[derive(Clone, Debug, PartialEq)]
pub struct Key {
    pub t: f64,
    pub anchor: Anchor,
    pub pos: [f64; 3],
    pub look: Look,
    /// degrees, added to an `Ahead` look (left and up are positive)
    pub yaw: f64,
    pub pitch: f64,
    /// degrees, clockwise as seen
    pub roll: f64,
    /// degrees added to play's field of view (as the player set it): 0
    /// keeps it, -8 narrows it
    pub fov: f64,
    /// how the segment into this key eases
    pub ease: Ease,
    /// a cut: the camera holds the last key and jumps to this one at `t`
    pub cut: bool,
    /// a held camera's sway (`curve::sway`), 0 for none
    pub sway: f64,
    /// the segment into this key is walked (0 to 1): a step's bob by the
    /// distance covered, as your own walk, faded out at both its ends
    pub walk: f64,
    /// the segment into this key is a jolt (a start, a fall): the comfort
    /// limits do not hold for it
    pub jolt: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Fade {
    pub t: f64,
    /// opacity reached (0 clear, 1 covered)
    pub to: f64,
    pub over: f64,
    pub colour: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Sound {
    pub t: f64,
    pub name: String,
    pub anchor: Anchor,
    pub pos: [f64; 3],
    pub gain: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Event {
    pub t: f64,
    pub name: String,
    /// run on a skip too (what it does to the world must not be lost)
    pub on_skip: bool,
}

/// An aside posted to your asides' director (`audio::asides`), said once
/// your hands are back if it is chosen
#[derive(Clone, Debug, PartialEq)]
pub struct Say {
    pub t: f64,
    pub use_: String,
    pub subject: String,
}

/// When the bars start to come in and to go
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bars {
    pub in_at: f64,
    pub out_at: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Cutscene {
    pub id: String,
    pub about: String,
    pub world: World,
    pub then: Then,
    pub skip: Skip,
    pub breaks_off: bool,
    pub length: f64,
    pub blend_in: f64,
    pub blend_out: f64,
    pub repeat_from: f64,
    pub stages: Vec<String>,
    /// events run once when your hands come back, however they do (played,
    /// skipped, broken off): what must follow it (a level card)
    pub finally: Vec<String>,
    pub bars: Bars,
    pub camera: Vec<Key>,
    pub fade: Vec<Fade>,
    pub sound: Vec<Sound>,
    pub event: Vec<Event>,
    pub say: Vec<Say>,
}

/// Reads one object's fields, each exactly once; anything left over is an
/// error (`done`)
struct Fields<'a> {
    at: String,
    m: &'a Map<String, Value>,
    used: Vec<&'static str>,
}

impl<'a> Fields<'a> {
    fn of(v: &'a Value, at: String) -> Result<Fields<'a>, String> {
        let m = v.as_object().ok_or_else(|| format!("{at}: not a table"))?;
        Ok(Fields { at, m, used: vec![] })
    }
    fn get(&mut self, k: &'static str) -> Result<&'a Value, String> {
        self.used.push(k);
        self.m.get(k).ok_or_else(|| format!("{}: no `{k}`", self.at))
    }
    fn num(&mut self, k: &'static str) -> Result<f64, String> {
        let at = self.at.clone();
        self.get(k)?.as_f64().filter(|x| x.is_finite()).ok_or_else(|| format!("{at}: `{k}` is not a number"))
    }
    fn text(&mut self, k: &'static str) -> Result<String, String> {
        let at = self.at.clone();
        self.get(k)?.as_str().map(String::from).ok_or_else(|| format!("{at}: `{k}` is not text"))
    }
    fn flag(&mut self, k: &'static str) -> Result<bool, String> {
        let at = self.at.clone();
        self.get(k)?.as_bool().ok_or_else(|| format!("{at}: `{k}` is not true or false"))
    }
    fn vec3(&mut self, k: &'static str) -> Result<Option<[f64; 3]>, String> {
        let at = self.at.clone();
        match self.get(k)? {
            Value::Null => Ok(None),
            Value::Array(a) if a.len() == 3 => {
                let mut p = [0.0; 3];
                for (i, x) in a.iter().enumerate() {
                    p[i] = x.as_f64().filter(|x| x.is_finite()).ok_or_else(|| format!("{at}: `{k}` holds a non-number"))?;
                }
                Ok(Some(p))
            }
            _ => Err(format!("{at}: `{k}` is not [x, y, z]")),
        }
    }
    fn list(&mut self, k: &'static str) -> Result<&'a Vec<Value>, String> {
        let at = self.at.clone();
        self.get(k)?.as_array().ok_or_else(|| format!("{at}: `{k}` is not a list"))
    }
    fn anchor(&mut self, k: &'static str) -> Result<Anchor, String> {
        let s = self.text(k)?;
        Anchor::parse(&s).ok_or_else(|| format!("{}: `{k}` \"{s}\" is not an anchor (start, eyes, thing, killer)", self.at))
    }
    fn done(self) -> Result<(), String> {
        match self.m.keys().find(|k| !self.used.contains(&k.as_str())) {
            Some(k) => Err(format!("{}: unknown field `{k}`", self.at)),
            None => Ok(()),
        }
    }
}

impl Cutscene {
    /// One cutscene of the compiled bundle
    pub fn from_json(v: &Value) -> Result<Cutscene, String> {
        let id = v.get("id").and_then(Value::as_str).unwrap_or("?").to_string();
        let mut f = Fields::of(v, format!("cutscene {id}"))?;
        let _ = f.text("id")?;
        let about = f.text("about")?;
        let world = match f.text("world")?.as_str() {
            "live" => World::Live,
            "held" => World::Held,
            w => return Err(format!("cutscene {id}: world \"{w}\" (live or held)")),
        };
        let then = match f.text("then")?.as_str() {
            "back" => Then::Back,
            "card" => Then::Card,
            w => return Err(format!("cutscene {id}: then \"{w}\" (back or card)")),
        };
        let skip = match f.text("skip")?.as_str() {
            "hold" => Skip::Hold,
            "never" => Skip::Never,
            w => return Err(format!("cutscene {id}: skip \"{w}\" (hold or never)")),
        };
        let breaks_off = f.flag("breaks_off")?;
        let length = f.num("length")?;
        let blend_in = f.num("blend_in")?;
        let blend_out = f.num("blend_out")?;
        let repeat_from = f.num("repeat_from")?;
        let stages = f
            .list("stages")?
            .iter()
            .map(|s| s.as_str().map(String::from).ok_or_else(|| format!("cutscene {id}: a stage is not text")))
            .collect::<Result<Vec<_>, _>>()?;
        let finally = f
            .list("finally")?
            .iter()
            .map(|s| s.as_str().map(String::from).ok_or_else(|| format!("cutscene {id}: a finally is not text")))
            .collect::<Result<Vec<_>, _>>()?;
        let bars = {
            let mut b = Fields::of(f.get("bars")?, format!("cutscene {id}: bars"))?;
            let bars = Bars { in_at: b.num("in")?, out_at: b.num("out")? };
            b.done()?;
            bars
        };
        let mut camera = vec![];
        for (i, k) in f.list("camera")?.iter().enumerate() {
            let mut kf = Fields::of(k, format!("cutscene {id}: camera key {}", i + 1))?;
            let t = kf.num("t")?;
            let anchor = kf.anchor("anchor")?;
            let pos = kf.vec3("pos")?.unwrap_or([0.0; 3]);
            let point = kf.vec3("look")?;
            let at = match kf.get("look_at")? {
                Value::Null => None,
                Value::String(s) => Some(Anchor::parse(s).ok_or_else(|| format!("cutscene {id}: camera key {}: look_at \"{s}\"", i + 1))?),
                _ => return Err(format!("cutscene {id}: camera key {}: look_at is not text", i + 1)),
            };
            let look = match (point, at) {
                (Some(_), Some(_)) => return Err(format!("cutscene {id}: camera key {}: both look and look_at", i + 1)),
                (Some(p), None) => Look::Point(p),
                (None, Some(a)) => Look::At(a),
                (None, None) => Look::Ahead,
            };
            let ease_name = kf.text("ease")?;
            let ease = Ease::parse(&ease_name).ok_or_else(|| format!("cutscene {id}: camera key {}: ease \"{ease_name}\"", i + 1))?;
            camera.push(Key {
                t,
                anchor,
                pos,
                look,
                yaw: kf.num("yaw")?,
                pitch: kf.num("pitch")?,
                roll: kf.num("roll")?,
                fov: kf.num("fov")?,
                ease,
                cut: kf.flag("cut")?,
                sway: kf.num("sway")?,
                walk: kf.num("walk")?,
                jolt: kf.flag("jolt")?,
            });
            kf.done()?;
        }
        let mut fade = vec![];
        for (i, x) in f.list("fade")?.iter().enumerate() {
            let mut xf = Fields::of(x, format!("cutscene {id}: fade {}", i + 1))?;
            fade.push(Fade { t: xf.num("t")?, to: xf.num("to")?, over: xf.num("over")?, colour: xf.text("colour")? });
            xf.done()?;
        }
        let mut sound = vec![];
        for (i, x) in f.list("sound")?.iter().enumerate() {
            let mut xf = Fields::of(x, format!("cutscene {id}: sound {}", i + 1))?;
            sound.push(Sound {
                t: xf.num("t")?,
                name: xf.text("name")?,
                anchor: xf.anchor("anchor")?,
                pos: xf.vec3("pos")?.unwrap_or([0.0; 3]),
                gain: xf.num("gain")?,
            });
            xf.done()?;
        }
        let mut event = vec![];
        for (i, x) in f.list("event")?.iter().enumerate() {
            let mut xf = Fields::of(x, format!("cutscene {id}: event {}", i + 1))?;
            event.push(Event { t: xf.num("t")?, name: xf.text("name")?, on_skip: xf.flag("on_skip")? });
            xf.done()?;
        }
        let mut say = vec![];
        for (i, x) in f.list("say")?.iter().enumerate() {
            let mut xf = Fields::of(x, format!("cutscene {id}: say {}", i + 1))?;
            say.push(Say { t: xf.num("t")?, use_: xf.text("use")?, subject: xf.text("subject")? });
            xf.done()?;
        }
        f.done()?;
        let c = Cutscene {
            id,
            about,
            world,
            then,
            skip,
            breaks_off,
            length,
            blend_in,
            blend_out,
            repeat_from,
            stages,
            finally,
            bars,
            camera,
            fade,
            sound,
            event,
            say,
        };
        let problems = c.shape_problems();
        if let Some(p) = problems.first() {
            return Err(format!("cutscene {}: {p}", c.id));
        }
        Ok(c)
    }

    /// What is wrong with its shape (times, envelope, keys), whatever the
    /// game around it: the rules a source must keep (`docs/CUTSCENES.md`,
    /// the design). `cargo xtask cutscene check` holds a source to the same
    /// list before it is compiled.
    pub fn shape_problems(&self) -> Vec<String> {
        let mut p = vec![];
        let len = self.length;
        if !(len > 0.0 && len <= 30.0) {
            p.push(format!("length {len}: a cutscene lasts more than 0 and at most 30 s"));
        }
        if self.blend_in < 0.0 || self.blend_out < 0.0 || self.blend_in + self.blend_out > len {
            p.push(format!("blend_in {} and blend_out {} must fit in its length {len}", self.blend_in, self.blend_out));
        }
        if self.then == Then::Back && self.blend_out < 0.25 {
            p.push(format!("then = \"back\" blends back into your eyes: blend_out {} is under 0.25 s", self.blend_out));
        }
        if !(0.0..len).contains(&self.repeat_from) {
            p.push(format!("repeat_from {} is not within its length", self.repeat_from));
        }
        if self.skip == Skip::Never && len > 3.0 {
            p.push(format!("skip = \"never\" holds your hands {len} s: only for 3 s at most (a death)"));
        }
        // a jolt is a moment, not a licence: each at most JOLT_MAX, and one
        // that gives your hands back at most JOLT_BACK of them in all
        let jolts: Vec<f64> = self.camera.windows(2).filter(|w| w[1].jolt).map(|w| w[1].t - w[0].t).collect();
        if let Some(j) = jolts.iter().find(|j| **j > JOLT_MAX + 1e-9) {
            p.push(format!("a jolt of {j:.2} s: a jolt is at most {JOLT_MAX} s"));
        }
        if self.then == Then::Back && jolts.iter().sum::<f64>() > JOLT_BACK + 1e-9 {
            p.push(format!("jolts of {:.2} s in all: one that gives your hands back has at most {JOLT_BACK} s", jolts.iter().sum::<f64>()));
        }
        if self.breaks_off && self.then == Then::Card {
            p.push("breaks_off on a cutscene that ends on the card (a death): nothing breaks off a death".into());
        }
        if self.breaks_off && self.world == World::Held {
            p.push("breaks_off is for a live world (in a held one nothing comes)".into());
        }
        if self.stages.is_empty() {
            p.push("no stage: tests and previews need somewhere to play it (debug::stage)".into());
        }
        if self.camera.is_empty() {
            p.push("no camera key".into());
        }
        if self.bars.in_at < 0.0 || self.bars.in_at > self.bars.out_at || self.bars.out_at > len {
            p.push(format!("bars in {} and out {} must be in order within its length", self.bars.in_at, self.bars.out_at));
        }
        let mut last = -1.0;
        for (i, k) in self.camera.iter().enumerate() {
            if k.t <= last || k.t < 0.0 || k.t > len {
                p.push(format!("camera key {} at {}: keys go forward in time within the length", i + 1, k.t));
            }
            last = k.t;
            if !(-30.0..=15.0).contains(&k.fov) {
                p.push(format!("camera key {}: fov {} (play's moved by -30° to +15°)", i + 1, k.fov));
            }
            if !(0.0..=3.0).contains(&k.sway) {
                p.push(format!("camera key {}: sway {} (0–3)", i + 1, k.sway));
            }
            if !(0.0..=1.0).contains(&k.walk) {
                p.push(format!("camera key {}: walk {} (0–1)", i + 1, k.walk));
            }
            if k.anchor == Anchor::Killer && k.pos != [0.0; 3] {
                p.push(format!("camera key {}: the killer is a point to look at, not a place to stand", i + 1));
            }
        }
        // starting on the keys, the first is the camera as it is
        if self.blend_in == 0.0 {
            match self.camera.first() {
                Some(k)
                    if k.t == 0.0
                        && k.anchor == Anchor::Start
                        && k.pos == [0.0; 3]
                        && k.look == Look::Ahead
                        && k.yaw == 0.0
                        && k.pitch == 0.0
                        && k.roll == 0.0
                        && k.fov == 0.0
                        && k.sway == 0.0
                        && !k.cut => {}
                _ => p.push(
                    "blend_in = 0 starts on the keys: the first key must be the camera as it is (t = 0, anchor = \"start\", nothing else)"
                        .into(),
                ),
            }
        }
        for (what, ts) in [
            ("fade", self.fade.iter().map(|x| x.t).collect::<Vec<_>>()),
            ("sound", self.sound.iter().map(|x| x.t).collect()),
            ("event", self.event.iter().map(|x| x.t).collect()),
            ("say", self.say.iter().map(|x| x.t).collect()),
        ] {
            if let Some(t) = ts.iter().find(|t| !(0.0..=len).contains(*t)) {
                p.push(format!("a {what} at {t} is outside its length"));
            }
            if ts.windows(2).any(|w| w[1] < w[0]) {
                p.push(format!("its {what}s are not in order of time"));
            }
        }
        if self.then == Then::Back {
            let v = self.veil_at(self.length).0;
            if v > 1e-9 {
                p.push(format!("then = \"back\" gives your eyes back clear: its fades leave a veil of {v:.2} at the end"));
            }
        }
        for x in &self.fade {
            if !(0.0..=1.0).contains(&x.to) || x.over < 0.0 {
                p.push(format!("fade at {}: to {} (0–1), over {} (≥ 0)", x.t, x.to, x.over));
            }
        }
        p
    }
}

/// Every cutscene in a compiled bundle (`{"cutscenes": [...]}`), in its order
pub fn parse_bundle(text: &str) -> Result<Vec<Cutscene>, String> {
    let v: Value = serde_json::from_str(text).map_err(|e| format!("the cutscene bundle is not JSON: {e}"))?;
    let mut f = Fields::of(&v, "the cutscene bundle".into())?;
    let _ = f.text("about")?;
    let list = f.list("cutscenes")?;
    let out = list.iter().map(Cutscene::from_json).collect::<Result<Vec<_>, _>>()?;
    f.done()?;
    let mut ids: Vec<&str> = out.iter().map(|c| c.id.as_str()).collect();
    ids.sort_unstable();
    if let Some(w) = ids.windows(2).find(|w| w[0] == w[1]) {
        return Err(format!("two cutscenes are called {}", w[0]));
    }
    Ok(out)
}
