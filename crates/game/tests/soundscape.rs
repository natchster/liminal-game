//! `audio::soundscape` against recorded reference data (`data/soundscape_ref_data.rs`):
//! the same rng draws per api call and matching reverb impulse responses
//! (`make_ir`).

mod data {
    include!("data/soundscape_ref_data.rs");
}
use data::*;
use engine::audio::AudioContext;
use game::audio::soundscape::*;
use std::cell::Cell;
use std::rc::Rc;

/// The LCG the data was recorded with, counting its draws.
struct Lcg {
    seed: Cell<u32>,
    draws: Cell<u64>,
}
impl Lcg {
    fn next(&self) -> f64 {
        self.draws.set(self.draws.get() + 1);
        let s = (self.seed.get() as u64 * 1664525 + 1013904223) as u32;
        self.seed.set(s);
        s as f64 / 4294967296.0
    }
}
fn lcg() -> (Rc<Lcg>, Rng) {
    let l = Rc::new(Lcg { seed: Cell::new(12345), draws: Cell::new(0) });
    let l2 = l.clone();
    (l, Rc::new(move || l2.next()))
}
/// The Loop's per-lap reverb (as in `modes::loop_rooms::loop_reverb`)
fn loop_reverb(k: i32) -> ReverbPreset {
    let kf = k as f64;
    ReverbPreset::new(0.5 + 0.4 * kf, 0.8 - 0.06 * kf, 0.1 + 0.045 * kf, 0.004 + 0.006 * kf, 6 - (k >> 1))
}
fn preset(name: &str) -> ReverbPreset {
    if name == "loop3" { loop_reverb(3) } else { *reverb_preset(name).unwrap() }
}

fn check_ir(name: &str, data: &[Vec<f32>], r: &IrRef) {
    assert_eq!(r.name, name);
    assert_eq!(data[0].len(), r.len, "{name}: length");
    for (c, (d, cr)) in data.iter().zip(r.chans.iter()).enumerate() {
        let (mut sum, mut sumsq, mut wsum, mut max_abs, mut arg_max) = (0.0f64, 0.0f64, 0.0f64, 0.0f64, 0usize);
        for (i, &v) in d.iter().enumerate() {
            let v = v as f64;
            sum += v;
            sumsq += v * v;
            wsum += v * ((i % 7) + 1) as f64;
            if v.abs() > max_abs {
                max_abs = v.abs();
                arg_max = i;
            }
        }
        let close =
            |a: f64, b: f64, what: &str| assert!((a - b).abs() <= 1e-6 * (1.0 + b.abs()), "{name} ch{c} {what}: got {a} vs recorded {b}");
        close(sum, cr.sum, "sum");
        close(sumsq, cr.sumsq, "sumsq");
        close(wsum, cr.wsum, "wsum");
        close(max_abs, cr.max_abs, "max");
        assert_eq!(arg_max, cr.arg_max, "{name} ch{c} argmax");
        for &(i, v) in cr.samples {
            assert!((d[i] - v).abs() <= 1e-6 * (1.0 + v.abs()), "{name} ch{c} sample {i}: got {} vs recorded {v}", d[i]);
        }
    }
}

/// `make_ir` alone, at both sample rates, no audio device involved.
#[test]
fn make_ir_matches_reference() {
    for r in REFS {
        let (l, rng) = lcg();
        for _ in 0..r.construct_draws {
            rng();
        }
        for ir in r.irs {
            let data = make_ir_data(&preset(ir.name), r.sr, &*rng);
            check_ir(ir.name, &data, ir);
        }
        let zones = r.steps.iter().find(|s| s.0 == "zones").unwrap().1;
        assert_eq!(l.draws.get(), zones, "draws after all IRs at {}", r.sr);
    }
}

/// The whole api on a real context: draws per call and the IRs it caches.
/// The zone reverbs draw their noise from streams of their own since R2b
/// (`ir_stream`, so a lazily built one draws nothing from the frame's): each
/// cached IR is the reference's algorithm on its own stream (its length the
/// reference's); the stream is then moved on by what the reverbs drew in the
/// reference, so every call after sees the reference's numbers and draws as
/// it drew.
#[test]
fn soundscape_draws_match_reference() {
    let ac = AudioContext::new_silent();
    ac.set_muted(true);
    let Some(r) = REFS.iter().find(|r| r.sr == ac.sample_rate()) else {
        eprintln!("no reference for sample rate {} — skipped", ac.sample_rate());
        return;
    };
    let out = ac.create_gain();
    out.connect(&ac.destination());
    let (l, rng) = lcg();
    let s = Soundscape::new(&ac, &out, SoundscapeOpts { rng: Some(rng), hrtf: None });
    assert_eq!(l.draws.get(), r.construct_draws, "construction draws");
    for ir in r.irs {
        if ir.name == "loop3" {
            s.define_zone("loop3", loop_reverb(3));
            s.set_zone("loop3", 1.2);
        } else {
            s.set_zone(ir.name, 1.2);
        }
        assert_eq!(s.zone().as_deref(), Some(ir.name));
        let b = s.ir_for(ir.name).unwrap();
        let own = std::cell::RefCell::new(ir_stream(ir.name));
        let want = make_ir_data(&preset(ir.name), r.sr, &|| own.borrow_mut().next());
        assert_eq!(b.channel(0).len(), ir.len, "{} is as long as the reference's", ir.name);
        for (c, w) in want.iter().enumerate() {
            assert!(b.channel(c).iter().zip(w).all(|(a, b)| a.to_bits() == b.to_bits()), "{} ch{c}: not its own stream's", ir.name);
        }
    }
    assert_eq!(l.draws.get(), r.construct_draws, "the zone reverbs draw nothing from the soundscape's stream");
    let ir_draws = r.steps.iter().find(|s| s.0 == "zones").unwrap().1 - r.construct_draws;
    let rng = s.rng();
    for _ in 0..ir_draws {
        rng();
    }
    let mut steps = r.steps.iter();
    let mut mark = |label: &str| {
        let (l2, d) = steps.next().unwrap();
        assert_eq!(*l2, label);
        assert_eq!(l.draws.get(), *d, "draws after {label}");
    };
    mark("zones");
    s.set_zone("nonexistent", 1.2);
    mark("setZone fallback lobby (cached)");
    assert_eq!(s.zone().as_deref(), Some("lobby"));
    for surf in ["carpet", "wetcarpet", "concrete", "tile", "office", "stair", "mud"] {
        s.footstep(surf, 0.8, None);
        mark(&format!("footstep {surf}"));
    }
    s.heartbeat(80.0, 4, 0.0, None);
    mark("heartbeat 80 4 0");
    s.heartbeat(92.0, 5, 1.0, None);
    mark("heartbeat 92 5 1");
    s.heartbeat(96.0, 4, 0.4, None);
    mark("heartbeat 96 4 0.4");
    s.knock(3, 0.0, 6.0, -3.0, 0.6);
    mark("knock");
    s.screech(1.0, 0.9);
    mark("screech");
    let v = s.create_child_voice(None);
    mark("create_child_voice");
    for p in ["sob", "please", "help", "downhere", "wail", "nope"] {
        v.say(p, Some(0.05), 1.0, 1.0);
        mark(&format!("say {p}"));
    }
    for i in 0..400 {
        s.hum.update(0.5, 0.0, 3.0, -1.0, if i % 3 != 0 { 1.0 } else { 0.2 });
    }
    mark("hum.update x400");
    s.burst(&s.world, 0.0, 0.1, "lowpass", 500.0, 1.0, 0.1, None, 0.002);
    mark("burst");
    s.tone(&s.world, 0.0, 0.1, 500.0, 0.1, "sine", None, 0.003);
    mark("tone");
    assert_eq!(l.seed.get(), r.final_seed);
}
