//! The replay file, `OCREPLAY` (`docs/engineering/REPLAYS.md`): the magic,
//! a version, then chunks, each `tag (4 bytes) | length (u32) | CRC-32 of
//! the body (u32) | body (deflated)`, little-endian:
//!
//! - `HEAD`: JSON: the build it was made by, when, its marks, the strings
//!   the frames name (keys, HUD targets, menu lines);
//! - then per segment: `ANCR` (JSON: its first frame and how it starts: a
//!   fresh process, or a floor's anchor), `FRMS` (its frames), `HASH` (the
//!   state after every 60th frame);
//! - `TELE` (JSON: telemetry rows and the director's log, for playtests);
//! - `END_` (empty): a file without it was cut short.
//!
//! A frame is its interval in clock ticks (the difference from the frame
//! before, zigzag varint) and its input events, each an op byte and its
//! payload: a look or a wheel on the 1/256 grid the window rounds them to
//! (`Quantizer`) as whole grid steps, anything else as the f64's bits, so
//! nothing a frame read is lost. Segments follow one another: the frames
//! of a recording are one run, and a floor's anchor is where a replay may
//! also begin.
use super::{FrameInput, InputEvent, StateHash};
use crate::session::anchor::Anchor;
use serde_json::{Value, json};

pub const MAGIC: &[u8; 8] = b"OCREPLAY";
pub const VERSION: u16 = 1;
/// The state is kept after every this many frames (`HASH`).
pub const HASH_EVERY: u64 = 60;
/// The grid a look or a wheel is rounded to where it enters: a 256th of a point.
pub const INPUT_GRID: f64 = 256.0;

/// How a segment starts.
#[derive(Clone, Debug, PartialEq)]
pub enum Start {
    /// A fresh process, as the game starts: its per-frame stream's seed,
    /// its launch flags, its settings and its view.
    Session { native_seed: u64, flags: String, settings: Value, view: [f64; 3] },
    /// A floor's anchor (`session::anchor`).
    Floor(Anchor),
}

/// A run of frames from one start.
#[derive(Clone, Debug, PartialEq)]
pub struct Segment {
    pub start: Start,
    /// the index of its first frame in the whole recording
    pub first_frame: u64,
    /// each frame's interval in clock ticks and its input
    pub frames: Vec<(u32, FrameInput)>,
    /// (frame index, the state after that frame)
    pub hashes: Vec<(u64, StateHash)>,
}

/// A recording: its header (the build, when, its marks), its segments and
/// what was measured on the way.
#[derive(Clone, Debug, PartialEq)]
pub struct ReplayFile {
    pub header: Value,
    pub segments: Vec<Segment>,
    pub telemetry: Value,
}

// ------------------------------------------------------------- bytes

fn varint(out: &mut Vec<u8>, mut v: u64) {
    while v >= 0x80 {
        out.push((v as u8) | 0x80);
        v >>= 7;
    }
    out.push(v as u8);
}
fn zigzag(v: i64) -> u64 {
    ((v << 1) ^ (v >> 63)) as u64
}
fn unzigzag(v: u64) -> i64 {
    ((v >> 1) as i64) ^ -((v & 1) as i64)
}

struct Reader<'a> {
    b: &'a [u8],
    at: usize,
}
impl Reader<'_> {
    fn u8(&mut self) -> Result<u8, String> {
        let v = *self.b.get(self.at).ok_or("a replay cut short")?;
        self.at += 1;
        Ok(v)
    }
    fn varint(&mut self) -> Result<u64, String> {
        let (mut v, mut shift) = (0u64, 0u32);
        loop {
            let b = self.u8()?;
            if shift > 63 {
                return Err("a varint too long".into());
            }
            v |= u64::from(b & 0x7f) << shift;
            if b & 0x80 == 0 {
                return Ok(v);
            }
            shift += 7;
        }
    }
    fn bytes(&mut self, n: usize) -> Result<&[u8], String> {
        let s = self.b.get(self.at..self.at + n).ok_or("a replay cut short")?;
        self.at += n;
        Ok(s)
    }
    fn u32(&mut self) -> Result<u32, String> {
        Ok(u32::from_le_bytes(self.bytes(4)?.try_into().unwrap_or_default()))
    }
    fn u64(&mut self) -> Result<u64, String> {
        Ok(u64::from_le_bytes(self.bytes(8)?.try_into().unwrap_or_default()))
    }
    fn f64(&mut self) -> Result<f64, String> {
        self.u64().map(f64::from_bits)
    }
    fn done(&self) -> bool {
        self.at >= self.b.len()
    }
}

/// CRC-32 (IEEE), as zip and PNG compute it.
pub fn crc32(b: &[u8]) -> u32 {
    let mut c = !0u32;
    for &x in b {
        c ^= u32::from(x);
        for _ in 0..8 {
            c = if c & 1 != 0 { (c >> 1) ^ 0xEDB8_8320 } else { c >> 1 };
        }
    }
    !c
}

fn chunk(out: &mut Vec<u8>, tag: &[u8; 4], body: &[u8]) {
    let z = miniz_oxide::deflate::compress_to_vec(body, 9);
    out.extend_from_slice(tag);
    out.extend_from_slice(&(z.len() as u32).to_le_bytes());
    out.extend_from_slice(&crc32(&z).to_le_bytes());
    out.extend_from_slice(&z);
}

// ------------------------------------------------------------- events

/// A look or wheel value as whole grid steps, when it lies on the grid.
fn on_grid(v: f64) -> Option<i64> {
    let q = v * INPUT_GRID;
    (q.fract() == 0.0 && q.abs() < 9.0e15).then_some(q as i64)
}

struct Strings(Vec<String>);
impl Strings {
    fn of(&mut self, s: &str) -> u64 {
        match self.0.iter().position(|x| x == s) {
            Some(i) => i as u64,
            None => {
                self.0.push(s.to_string());
                (self.0.len() - 1) as u64
            }
        }
    }
}

fn put_event(out: &mut Vec<u8>, e: &InputEvent, strings: &mut Strings) {
    match e {
        InputEvent::Down(k) => {
            out.push(0);
            varint(out, strings.of(k));
        }
        InputEvent::Up(k) => {
            out.push(1);
            varint(out, strings.of(k));
        }
        InputEvent::Look(x, y) => match (on_grid(*x), on_grid(*y)) {
            (Some(qx), Some(qy)) => {
                out.push(2);
                varint(out, zigzag(qx));
                varint(out, zigzag(qy));
            }
            _ => {
                out.push(3);
                out.extend_from_slice(&x.to_bits().to_le_bytes());
                out.extend_from_slice(&y.to_bits().to_le_bytes());
            }
        },
        InputEvent::Click(h) => {
            out.push(4);
            varint(out, strings.of(h));
        }
        InputEvent::Lock(on) => {
            out.push(5);
            out.push(u8::from(*on));
        }
        InputEvent::Blur => out.push(6),
        InputEvent::Wheel(d) => match on_grid(*d) {
            Some(q) => {
                out.push(7);
                varint(out, zigzag(q));
            }
            None => {
                out.push(8);
                out.extend_from_slice(&d.to_bits().to_le_bytes());
            }
        },
        InputEvent::Hover(h) => {
            out.push(9);
            varint(out, strings.of(h));
        }
        InputEvent::View(w, h, s) => {
            out.push(10);
            for v in [w, h, s] {
                out.extend_from_slice(&v.to_bits().to_le_bytes());
            }
        }
    }
}

fn get_event(r: &mut Reader, strings: &[String]) -> Result<InputEvent, String> {
    let s = |r: &mut Reader| -> Result<String, String> {
        let i = r.varint()? as usize;
        strings.get(i).cloned().ok_or_else(|| format!("a string {i} the header does not have"))
    };
    let grid = |q: u64| unzigzag(q) as f64 / INPUT_GRID;
    Ok(match r.u8()? {
        0 => InputEvent::Down(s(r)?),
        1 => InputEvent::Up(s(r)?),
        2 => InputEvent::Look(grid(r.varint()?), grid(r.varint()?)),
        3 => InputEvent::Look(r.f64()?, r.f64()?),
        4 => InputEvent::Click(s(r)?),
        5 => InputEvent::Lock(r.u8()? != 0),
        6 => InputEvent::Blur,
        7 => InputEvent::Wheel(grid(r.varint()?)),
        8 => InputEvent::Wheel(r.f64()?),
        9 => InputEvent::Hover(s(r)?),
        10 => InputEvent::View(r.f64()?, r.f64()?, r.f64()?),
        op => return Err(format!("an input op {op} this game does not know")),
    })
}

// ------------------------------------------------------------- starts

impl Start {
    pub fn to_json(&self) -> Value {
        match self {
            Start::Session { native_seed, flags, settings, view } => json!({
                "kind": "session",
                "native_seed": native_seed,
                "flags": flags,
                "settings": settings,
                "view": view.map(f64::to_bits),
            }),
            Start::Floor(a) => json!({ "kind": "floor", "anchor": a.to_json() }),
        }
    }
    pub fn from_json(v: &Value) -> Result<Start, String> {
        match v["kind"].as_str() {
            Some("session") => {
                let view: Vec<f64> = v["view"].as_array().into_iter().flatten().filter_map(Value::as_u64).map(f64::from_bits).collect();
                Ok(Start::Session {
                    native_seed: v["native_seed"].as_u64().ok_or("a session start without its seed")?,
                    flags: v["flags"].as_str().unwrap_or_default().to_string(),
                    settings: v["settings"].clone(),
                    view: <[f64; 3]>::try_from(view).map_err(|_| "a session start without its view")?,
                })
            }
            Some("floor") => Ok(Start::Floor(Anchor::from_json(&v["anchor"]).ok_or("a floor start whose anchor does not read")?)),
            k => Err(format!("a start of kind {k:?}")),
        }
    }
}

// ------------------------------------------------------------- the file

impl ReplayFile {
    /// Every frame of the recording, in order, with its index.
    pub fn frame_count(&self) -> u64 {
        self.segments.iter().map(|s| s.frames.len() as u64).sum()
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut strings = Strings(vec![]);
        let mut bodies = vec![];
        for s in &self.segments {
            let mut f = vec![];
            varint(&mut f, s.frames.len() as u64);
            let mut prev = 0u32;
            for (ticks, input) in &s.frames {
                varint(&mut f, zigzag(i64::from(*ticks) - i64::from(prev)));
                prev = *ticks;
                varint(&mut f, input.events.len() as u64);
                for e in &input.events {
                    put_event(&mut f, e, &mut strings);
                }
            }
            let mut h = vec![];
            varint(&mut h, s.hashes.len() as u64);
            for (k, sh) in &s.hashes {
                varint(&mut h, *k);
                for part in [sh.player, sh.entities, sh.director, sh.world] {
                    h.extend_from_slice(&part.to_le_bytes());
                }
            }
            let a = json!({ "first_frame": s.first_frame, "start": s.start.to_json() });
            bodies.push((a.to_string().into_bytes(), f, h));
        }
        let mut header = self.header.clone();
        header["format"] = json!(VERSION);
        header["strings"] = json!(strings.0);
        header["frames"] = json!(self.frame_count());
        header["hash_every"] = json!(HASH_EVERY);
        let mut out = MAGIC.to_vec();
        out.extend_from_slice(&VERSION.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        chunk(&mut out, b"HEAD", header.to_string().as_bytes());
        for (a, f, h) in bodies {
            chunk(&mut out, b"ANCR", &a);
            chunk(&mut out, b"FRMS", &f);
            chunk(&mut out, b"HASH", &h);
        }
        chunk(&mut out, b"TELE", self.telemetry.to_string().as_bytes());
        chunk(&mut out, b"END_", &[]);
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<ReplayFile, String> {
        let mut r = Reader { b: bytes, at: 0 };
        if r.bytes(8)? != MAGIC {
            return Err("not a replay (no OCREPLAY)".into());
        }
        let version = u16::from_le_bytes(r.bytes(2)?.try_into().unwrap_or_default());
        if version > VERSION {
            return Err(format!("a replay of format {version}, newer than this game's {VERSION}"));
        }
        r.bytes(2)?;
        let (mut header, mut segments, mut telemetry, mut ended) = (Value::Null, Vec::<Segment>::new(), json!({}), false);
        let mut strings: Vec<String> = vec![];
        while !r.done() {
            let tag: [u8; 4] = r.bytes(4)?.try_into().unwrap_or_default();
            let len = r.u32()? as usize;
            let crc = r.u32()?;
            let z = r.bytes(len)?;
            if crc32(z) != crc {
                return Err(format!("the chunk {} is damaged", String::from_utf8_lossy(&tag)));
            }
            let body =
                miniz_oxide::inflate::decompress_to_vec(z).map_err(|e| format!("the chunk {}: {e:?}", String::from_utf8_lossy(&tag)))?;
            let json = || serde_json::from_slice::<Value>(&body).map_err(|e| format!("{}: {e}", String::from_utf8_lossy(&tag)));
            match &tag {
                b"HEAD" => {
                    header = json()?;
                    strings = header["strings"].as_array().into_iter().flatten().filter_map(|s| s.as_str().map(String::from)).collect();
                }
                b"ANCR" => {
                    let a = json()?;
                    segments.push(Segment {
                        start: Start::from_json(&a["start"])?,
                        first_frame: a["first_frame"].as_u64().ok_or("a segment without its first frame")?,
                        frames: vec![],
                        hashes: vec![],
                    });
                }
                b"FRMS" => {
                    let seg = segments.last_mut().ok_or("frames before any start")?;
                    let mut b = Reader { b: &body, at: 0 };
                    let n = b.varint()?;
                    let mut prev = 0i64;
                    for _ in 0..n {
                        prev += unzigzag(b.varint()?);
                        let ticks = u32::try_from(prev).map_err(|_| "a frame's interval out of range")?;
                        let k = b.varint()?;
                        let events = (0..k).map(|_| get_event(&mut b, &strings)).collect::<Result<Vec<_>, _>>()?;
                        seg.frames.push((ticks, FrameInput { events }));
                    }
                }
                b"HASH" => {
                    let seg = segments.last_mut().ok_or("hashes before any start")?;
                    let mut b = Reader { b: &body, at: 0 };
                    for _ in 0..b.varint()? {
                        let k = b.varint()?;
                        let sh = StateHash { player: b.u64()?, entities: b.u64()?, director: b.u64()?, world: b.u64()? };
                        seg.hashes.push((k, sh));
                    }
                }
                b"TELE" => telemetry = json()?,
                b"END_" => ended = true,
                // a newer chunk this game does not know: skipped
                _ => {}
            }
        }
        if !ended {
            return Err("a replay cut short (no END_)".into());
        }
        if header.is_null() {
            return Err("a replay without its header".into());
        }
        Ok(ReplayFile { header, segments, telemetry })
    }

    /// The whole file as JSON (`--dump-replay`).
    pub fn to_json(&self) -> Value {
        json!({
            "header": self.header,
            "segments": self.segments.iter().map(|s| json!({
                "first_frame": s.first_frame,
                "start": s.start.to_json(),
                "frames": s.frames.iter().map(|(t, i)| json!([t, i.to_json()])).collect::<Vec<_>>(),
                "hashes": s.hashes.iter().map(|(k, h)| json!([k, h.player, h.entities, h.director, h.world])).collect::<Vec<_>>(),
            })).collect::<Vec<_>>(),
            "telemetry": self.telemetry,
        })
    }
}

/// Rounds the window's look and wheel to `INPUT_GRID` where they enter,
/// carrying what rounding left to the next one (as Doom carries its mouse),
/// so slow moves are not lost and what the game reads is exactly what a
/// recording keeps.
#[derive(Clone, Copy, Debug, Default)]
pub struct Quantizer {
    look: (f64, f64),
    wheel: f64,
}
impl Quantizer {
    pub fn look(&mut self, dx: f64, dy: f64) -> (f64, f64) {
        let (x, y) = (dx * INPUT_GRID + self.look.0, dy * INPUT_GRID + self.look.1);
        let (qx, qy) = (x.round(), y.round());
        self.look = (x - qx, y - qy);
        (qx / INPUT_GRID, qy / INPUT_GRID)
    }
    pub fn wheel(&mut self, d: f64) -> f64 {
        let x = d * INPUT_GRID + self.wheel;
        let q = x.round();
        self.wheel = x - q;
        q / INPUT_GRID
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn varints_and_zigzag_round_trip() {
        for v in [0i64, 1, -1, 63, -64, 1 << 40, -(1 << 40), i64::MAX >> 1, i64::MIN >> 1] {
            let mut b = vec![];
            varint(&mut b, zigzag(v));
            assert_eq!(unzigzag(Reader { b: &b, at: 0 }.varint().unwrap()), v);
        }
    }

    #[test]
    fn the_crc_is_ieees() {
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
    }

    #[test]
    fn rounding_carries_what_it_leaves() {
        let mut q = Quantizer::default();
        // 300 moves of a 1000th of a point: each rounds to nothing alone,
        // and the carry turns them into whole steps
        let total: f64 = (0..300).map(|_| q.look(0.001, 0.0).0).sum();
        assert!((total - 0.3).abs() <= 1.0 / INPUT_GRID, "{total}");
        assert_eq!(q.look(1.5, -2.25), (1.5, -2.25));
    }

    #[test]
    fn a_file_round_trips_whatever_its_events() {
        let mut i = FrameInput::default();
        i.down("KeyW");
        i.look(0.5, -0.25);
        i.events.push(InputEvent::Look(0.1, 0.3)); // off the grid: kept as its bits
        i.events.push(InputEvent::Click("menu:start".into()));
        i.events.push(InputEvent::Lock(true));
        i.events.push(InputEvent::Blur);
        i.events.push(InputEvent::Wheel(1.0 / 256.0));
        i.events.push(InputEvent::Wheel(0.3));
        i.events.push(InputEvent::Hover("continue".into()));
        i.events.push(InputEvent::View(1280.0, 800.0, 2.0));
        i.up("KeyW");
        let seg = Segment {
            start: Start::Session { native_seed: 42, flags: "seed=7".into(), settings: json!({"look": 1.0}), view: [1280.0, 800.0, 2.0] },
            first_frame: 0,
            frames: vec![(1092, i.clone()), (1093, FrameInput::default()), (0, i)],
            hashes: vec![(59, StateHash { player: 1, entities: 2, director: 3, world: u64::MAX })],
        };
        let f = ReplayFile { header: json!({ "title": "test" }), segments: vec![seg], telemetry: json!({ "rows": [] }) };
        let bytes = f.encode();
        let back = ReplayFile::decode(&bytes).unwrap();
        assert_eq!(back.segments, f.segments);
        assert_eq!(back.header["title"], "test");
        // damage and truncation are found
        let mut bad = bytes.clone();
        let mid = bad.len() / 2;
        bad[mid] ^= 0x55;
        assert!(ReplayFile::decode(&bad).is_err());
        assert!(ReplayFile::decode(&bytes[..bytes.len() - 13]).is_err());
    }
}
