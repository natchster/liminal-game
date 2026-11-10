//! Listening, headless: levels are played with the audio graph rendered
//! offline, recorded to WAV under `target/listen/` (to listen to) with a
//! loudness timeline next to each (PNG), and measured in 50 ms windows:
//!
//! * a level opens quietly: nothing piles up in its first second;
//! * the moment a creature notices you is loud against what came before;
//! * the director's hush quiets the ambience: fewer and softer events, a
//!   lower room tone;
//! * what must be heard anywhere is heard beside a lamp too, where the
//!   ballast's hum is the bed: measured in the band it is heard in
//!   (`band_passed`; over the whole mix the hum's lines outweigh it).
//!
//!   cargo test --release -p backrooms --test listen -- --nocapture
use engine::audio::AudioContext;
use game::Game;

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

const SR: f64 = 48000.0;
/// 50 ms
const WIN: usize = 2400;

struct Run {
    g: Game,
    ac: AudioContext,
    /// interleaved stereo
    pcm: Vec<f32>,
}

fn start(seed: u32, level: i32) -> Run {
    let mut g = Game::new_bare();
    register_placers(&mut g.placers);
    g.launch_flags = format!("seed={seed}&level={level}");
    let ac = AudioContext::new_offline(SR);
    g.audio_init_with(ac.clone());
    g.boot();
    g.start_new_game();
    g.god_mode = true;
    Run { g, ac, pcm: vec![] }
}
/// `secs` of play at 60 fps, `each` called before every frame
fn play(r: &mut Run, secs: f64, mut each: impl FnMut(&mut Game)) {
    for _ in 0..(secs * 60.0).round() as usize {
        each(&mut r.g);
        r.g.now_ms += 1000.0 / 60.0;
        r.g.fixed_step_dt = 1.0 / 60.0;
        r.g.frame(1.0 / 60.0);
        r.g.fixed_step_dt = 0.0;
        r.pcm.extend(r.ac.render_offline((SR / 60.0) as usize));
    }
}
/// Every creature stands still and silent where it is.
fn hold_creatures(g: &mut Game) {
    for e in g.entities.iter_mut() {
        e.freeze = 1e9;
    }
}
/// Loudness of each 50 ms window (RMS of the mono mix)
fn windows(pcm: &[f32]) -> Vec<f64> {
    pcm.chunks(WIN * 2)
        .map(|c| {
            let n = c.len() / 2;
            let s: f64 = (0..n).map(|i| ((c[2 * i] + c[2 * i + 1]) as f64 * 0.5).powi(2)).sum();
            (s / n.max(1) as f64).sqrt()
        })
        .collect()
}
/// The power of the mix between `lo` and `hi` Hz in each 50 ms window (a
/// Hann-windowed DFT of those bins): what a whisper adds, over a bed that
/// is mostly hum and room below it and the grain's hiss above it
fn windows_band(pcm: &[f32], lo: f64, hi: f64) -> Vec<f64> {
    let n = WIN;
    let hann: Vec<f64> = (0..n).map(|i| 0.5 - 0.5 * (2.0 * std::f64::consts::PI * i as f64 / n as f64).cos()).collect();
    let bins: Vec<usize> = (0..n / 2).filter(|&k| (lo..hi).contains(&(k as f64 * SR / n as f64))).collect();
    let tw: Vec<Vec<(f64, f64)>> = bins
        .iter()
        .map(|&k| (0..n).map(|i| (2.0 * std::f64::consts::PI * (k * i) as f64 / n as f64).sin_cos()).map(|(s, c)| (c, s)).collect())
        .collect();
    pcm.chunks(WIN * 2)
        .filter(|c| c.len() == WIN * 2)
        .map(|c| {
            let x: Vec<f64> = (0..n).map(|i| (c[2 * i] + c[2 * i + 1]) as f64 * 0.5 * hann[i]).collect();
            let p: f64 = tw
                .iter()
                .map(|t| {
                    let (re, im) = x.iter().zip(t).fold((0.0, 0.0), |(re, im), (v, (c, s))| (re + v * c, im - v * s));
                    re * re + im * im
                })
                .sum();
            (p / n as f64).sqrt()
        })
        .collect()
}
/// The power of the mix between `lo` and `hi` Hz in each half second from
/// `from` to `to` (samples a channel; 2 Hz bins): finer than `windows_band`,
/// for a band a held note fills among the room's own lines (its drone and
/// the lamps lay a line every 47 and every 120 Hz)
fn band_power(pcm: &[f32], from: usize, to: usize, lo: f64, hi: f64) -> Vec<f64> {
    let n = (SR * 0.5) as usize;
    let hann: Vec<f64> = (0..n).map(|i| 0.5 - 0.5 * (2.0 * std::f64::consts::PI * i as f64 / n as f64).cos()).collect();
    let bins: Vec<usize> = (0..n / 2).filter(|&k| (lo..hi).contains(&(k as f64 * SR / n as f64))).collect();
    let mut out = vec![];
    let mut at = from;
    while at + n <= to.min(pcm.len() / 2) {
        let x: Vec<f64> = (0..n).map(|i| (pcm[2 * (at + i)] + pcm[2 * (at + i) + 1]) as f64 * 0.5 * hann[i]).collect();
        let p: f64 = bins
            .iter()
            .map(|&k| {
                let w = 2.0 * std::f64::consts::PI * k as f64 / n as f64;
                let (re, im) = x
                    .iter()
                    .enumerate()
                    .fold((0.0, 0.0), |(re, im), (i, v)| (re + v * (w * i as f64).cos(), im - v * (w * i as f64).sin()));
                re * re + im * im
            })
            .sum();
        out.push(p);
        at += n;
    }
    out
}
fn quantile(v: &[f64], q: f64) -> f64 {
    let mut s = v.to_vec();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap());
    s[((s.len() - 1) as f64 * q) as usize]
}
fn median(v: &[f64]) -> f64 {
    let mut s = v.to_vec();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap());
    s[s.len() / 2]
}
/// How much of the time something stands out of the bed: the summed
/// loudness above twice the median, per second
fn event_energy(w: &[f64], bed: f64) -> f64 {
    w.iter().map(|v| (v - 2.0 * bed).max(0.0)).sum::<f64>() / (w.len() as f64 * 0.05)
}
fn db(x: f64) -> f64 {
    20.0 * x.max(1e-7).log10()
}
/// The WAV and a loudness timeline (dB, -70..0, one column per window; red
/// marks at `marks` seconds)
fn save(name: &str, pcm: &[f32], marks: &[f64]) {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/listen");
    std::fs::create_dir_all(&dir).unwrap();
    let base = dir.join(name);
    engine::wav::write_stereo(base.with_extension("wav").to_str().unwrap(), SR as u32, pcm).unwrap();
    let w = windows(pcm);
    let (wd, h) = (w.len().max(1), 160usize);
    let mut px = vec![0u8; wd * h * 4];
    for (x, v) in w.iter().enumerate() {
        let top = ((db(*v) / -70.0).clamp(0.0, 1.0) * h as f64) as usize;
        for y in 0..h {
            let i = (y * wd + x) * 4;
            let on = y >= top;
            let grid = y % 40 == 0;
            px[i..i + 4].copy_from_slice(&if on {
                [210, 200, 150, 255]
            } else if grid {
                [40, 40, 40, 255]
            } else {
                [16, 16, 16, 255]
            });
        }
    }
    for m in marks {
        let x = (m / 0.05) as usize;
        for y in 0..h {
            if x < wd {
                let i = (y * wd + x) * 4;
                px[i..i + 4].copy_from_slice(&[220, 40, 40, 255]);
            }
        }
    }
    engine::png::write_rgba(base.with_extension("png").to_str().unwrap(), wd as u32, h as u32, &px).unwrap();
}

#[test]
fn a_level_opens_quietly() {
    for (seed, level) in [(4242u32, 6), (77, 8), (9001, 11)] {
        let mut r = start(seed, level);
        play(&mut r, 90.0, hold_creatures);
        let w = windows(&r.pcm);
        let bed = median(&w[40..]);
        let first = w[..20].iter().cloned().fold(0.0, f64::max);
        let later = w[40..].iter().cloned().fold(0.0, f64::max);
        println!(
            "seed {seed} L{level}: bed {:.1} dB, loudest in the first second {:.1} dB, loudest after {:.1} dB, events {:.4}/s",
            db(bed),
            db(first),
            db(later),
            event_energy(&w[40..], bed)
        );
        save(&format!("open_{seed}_L{level}"), &r.pcm, &[1.0]);
        assert!(first < bed * 2.0, "seed {seed} L{level}: the first second piles up ({:.1} dB over the bed)", db(first / bed));
        assert!(later > bed * 2.0, "seed {seed} L{level}: in a minute and a half nothing stands out of the bed");
    }
}

/// Each signature sound, when it comes, stands out of the bed: you hear
/// that something happened somewhere, not a level that never makes a sound.
#[test]
fn signatures_are_heard() {
    let mut heard = vec![];
    // (six floors: three were a sample the claim held on by luck)
    for (seed, level) in [(4242u32, 6), (77, 8), (9001, 11), (1999, 10), (31337, 8), (7, 7)] {
        // (the per-frame stream seeded for each run: the tests run side by
        // side, and a shared stream made the signatures' times depend on
        // what the others drew)
        game::base::rng::seed_native(0x115E_0000 ^ u64::from(seed) ^ ((level as u64) << 32));
        let mut r = start(seed, level);
        let mut times = vec![];
        let mut last = String::new();
        let mut t = 0.0;
        play(&mut r, 150.0, |g| {
            hold_creatures(g);
            t += 1.0 / 60.0;
            if let Some(e) = g.ambient_log.last() {
                let key = format!("{}@{}", e.tag, e.at);
                if key != last {
                    last = key;
                    if e.tag.starts_with("sig-") {
                        // whether nothing stands between you and where it sounded
                        let name = e.tag.trim_start_matches("sig-");
                        let p = g.player.pos;
                        let clear = g.amb_signatures.iter().find(|s| s.name == name).is_some_and(|s| g.los(p.x, p.z, s.at.x, s.at.z));
                        times.push((t, e.tag.clone(), e.dist.unwrap_or(0.0), clear));
                    }
                }
            }
        });
        let w = windows(&r.pcm);
        let bed = median(&w[40..]);
        for (at, tag, dist, clear) in &times {
            let a = (at / 0.05) as usize;
            let peak = w[a.min(w.len() - 1)..(a + 40).min(w.len())].iter().cloned().fold(0.0, f64::max);
            let how = if *clear { "in the open" } else { "through walls" };
            println!("seed {seed} L{level}: {tag} at {at:.1}s, {dist:.0} m off, {how}: +{:.1} dB over the bed", db(peak / bed));
            heard.push((peak / bed, *clear));
        }
        save(&format!("signatures_{seed}_L{level}"), &r.pcm, &times.iter().map(|x| x.0).collect::<Vec<_>>());
    }
    // Each lands some 6–14 dB over the bed at its distance, and walls muffle
    // it (`audio::ambient::sig_carry`): most of those with nothing in the way
    // rise 6 dB, and well over half of them all do. (The claim was "most of
    // all of them", and held on three floors; on six, what stayed under was
    // a thud or a clank three walls away.)
    assert!(heard.len() >= 6, "signatures come round: {} in 6 × 150 s", heard.len());
    let open: Vec<f64> = heard.iter().filter(|h| h.1).map(|h| h.0).collect();
    let loud_open = open.iter().filter(|r| **r >= 2.0).count();
    let loud = heard.iter().filter(|h| h.0 >= 2.0).count();
    println!("{loud_open} of {} in the open and {loud} of {} in all rise 6 dB over the bed", open.len(), heard.len());
    assert!(open.len() >= 6, "signatures in the open: {}", open.len());
    assert!(loud_open * 4 >= open.len() * 3, "most signatures in the open rise 6 dB over the bed: {loud_open} of {}", open.len());
    assert!(loud * 5 >= heard.len() * 3, "well over half of all signatures rise 6 dB over the bed: {loud} of {}", heard.len());
}

#[test]
fn the_notice_is_loud() {
    // a smiler 9 m ahead, held while the room's bed is measured, your lamp
    // off; then your lamp finds it: the click of its teeth, then its hiss
    // when it is sure where you are
    let mut r = start(4242, 6);
    hold_creatures(&mut r.g);
    let ei = {
        let g = &mut r.g;
        let f = g.player.fwd();
        let (x, z) = (g.player.pos.x + f.x * 9.0, g.player.pos.z + f.z * 9.0);
        let (tx, tz) = (g.tx(x), g.tz(z));
        let ei = g.add_entity("smiler", tx, tz, 0);
        g.entities[ei].g.set_pos(x, 0.0, z);
        ei
    };
    play(&mut r, 6.0, |g| {
        g.entities[ei].freeze = 1e9;
        g.player.flashlight_on = false;
    });
    let before = r.pcm.len() / 2;
    play(&mut r, 4.0, |g| {
        g.entities[ei].freeze = 0.0;
        g.player.flashlight_on = true;
    });
    let log: Vec<String> =
        r.g.creature(ei).map(|c| c.brain.log.iter().map(|x| format!("{:.1} {} ({})", x.0, x.2.name(), x.3)).collect()).unwrap_or_default();
    println!("smiler log: {log:?}");
    let noticed = r.g.creature(ei).map(|c| c.brain.log.iter().any(|x| x.2 == game::entities::brain::Mode::Hunt)).unwrap_or(false);
    assert!(noticed, "it noticed your lamp: {log:?}");
    let w = windows(&r.pcm);
    let b = before / WIN;
    let calm = median(&w[40..b]);
    let peak = w[b..(b + 50).min(w.len())].iter().cloned().fold(0.0, f64::max);
    println!("notice: before {:.1} dB, peak after {:.1} dB (+{:.1} dB)", db(calm), db(peak), db(peak / calm));
    save("notice_4242_L3", &r.pcm, &[before as f64 / SR]);
    assert!(peak > calm * 4.0, "the notice should stand 12 dB over the quiet before it");
}

/// Your cry is your own voice (`docs/VOICES.md`): a line said in the
/// place you stand, standing far over the room, and never clipping (the
/// formant cry it replaced hit 0 dBFS; it is measured beside it)
#[test]
fn the_cry_is_your_voice() {
    // the room, then a cry, measured where a voice is (200 Hz - 3 kHz)
    let hear = |say: fn(&mut Game) -> bool| {
        let mut r = start(4242, 5);
        hold_creatures(&mut r.g);
        play(&mut r, 3.0, hold_creatures);
        let at = r.pcm.len() / 2;
        assert!(say(&mut r.g), "nothing was said");
        play(&mut r, 3.0, hold_creatures);
        let w = windows_band(&r.pcm, 200.0, 3000.0);
        let b = at / WIN;
        let bed = median(&w[20..b]);
        let peak = w[b..].iter().cloned().fold(0.0, f64::max);
        let line = r.g.cry_last_line.clone();
        let top = r.pcm[at * 2..].iter().fold(0f32, |m, v| m.max(v.abs())) as f64;
        (bed, peak, r.pcm, at as f64 / SR, line, top)
    };
    let (bed, peak, pcm, at, line, top) = hear(|g| g.say_cry());
    let (fbed, fpeak, fpcm, fat, _, ftop) = hear(|g| {
        g.sfx_cry();
        true
    });
    save("cry_voice_4242_L5", &pcm, &[at]);
    save("cry_formant_4242_L5", &fpcm, &[fat]);
    println!(
        "cry: the line {line:?}, {:.1} dB over the room, peak {:.1} dBFS; the formant cry {:.1} dB over it, peak {:.1} dBFS",
        db(peak / bed),
        db(top),
        db(fpeak / fbed),
        db(ftop)
    );
    assert!(line.is_some(), "the cry said no line");
    assert!(peak > bed * 10.0, "your cry should stand 20 dB over the room ({:.1} dB)", db(peak / bed));
    assert!(db(top) <= -1.0, "your cry must not clip ({:.1} dBFS)", db(top));
}

/// Your asides (`docs/VOICES.md`, *Your asides*): the run's first floor,
/// once its card and line have gone and you walk on, brings its arrival,
/// your own voice under your breath. Measured where a voice is (200 Hz -
/// 3 kHz): over the room and over your own steps, under your cry on the
/// same floor, and never clipping. (With `ASIDE_GAIN` at 0 it fails: tried
/// once, A4.)
#[test]
fn an_arrival_is_your_voice_under_your_breath() {
    use game::audio::asides::ARRIVAL_AFTER;
    let mut r = start(4242, 5);
    // the card and the floor's line, standing: nothing is said
    play(&mut r, ARRIVAL_AFTER + 0.5, hold_creatures);
    assert!(!r.g.asides.log.iter().any(|(_, w)| w.starts_with("say:")), "said before you walked: {:?}", r.g.asides.log);
    let at = r.pcm.len() / 2;
    // (the voice director's clock: its log's times)
    let t_at = r.g.asides.clock;
    play(&mut r, 7.0, |g| {
        hold_creatures(g);
        g.keys.insert("KeyW".into());
    });
    let (t_said, what) = r.g.asides.log.iter().find(|(_, w)| w.starts_with("say:arrive:")).cloned().expect("no arrival said");
    let said = at + ((t_said - t_at) * SR) as usize;
    let w = windows_band(&r.pcm, 200.0, 3000.0);
    let (b, s) = (at / WIN, said / WIN);
    let bed = median(&w[20..b]);
    // your steps before you spoke, and your voice in the 1.5 s after
    let steps = w[b..s].iter().cloned().fold(0.0, f64::max);
    let voice = w[s..(s + 30).min(w.len())].iter().cloned().fold(0.0, f64::max);
    let top = r.pcm[said * 2..].iter().fold(0f32, |m, v| m.max(v.abs())) as f64;
    save("aside_arrive_4242_L5", &r.pcm, &[at as f64 / SR, said as f64 / SR]);
    // your cry on the same floor, measured the same way
    let mut c = start(4242, 5);
    play(&mut c, 3.0, hold_creatures);
    let cat = c.pcm.len() / 2;
    assert!(c.g.say_cry());
    play(&mut c, 3.0, hold_creatures);
    let cw = windows_band(&c.pcm, 200.0, 3000.0);
    let cry = db(cw[cat / WIN..].iter().cloned().fold(0.0, f64::max) / median(&cw[20..cat / WIN]));
    println!(
        "{what}, {:.1} s after the first steps: {:.1} dB over the room, {:.1} dB over your steps, peak {:.1} dBFS; your cry {cry:.1} dB over the room",
        t_said - t_at,
        db(voice / bed),
        db(voice / steps),
        db(top)
    );
    assert!(voice > bed * 4.0, "an aside should stand 12 dB over the room ({:.1} dB)", db(voice / bed));
    assert!(voice > steps, "an aside should stand over your own steps ({:.1} dB)", db(voice / steps));
    assert!(db(voice / bed) < cry, "an aside is under your breath: under your cry ({:.1} dB, the cry {cry:.1} dB)", db(voice / bed));
    assert!(db(top) <= -1.0, "an aside must not clip ({:.1} dBFS)", db(top));
}

#[test]
fn the_hush_quiets_the_ambience() {
    let mut out: Vec<(f64, Vec<f64>)> = vec![];
    for hushed in [false, true] {
        let mut r = start(77, 8);
        play(&mut r, 150.0, |g| {
            hold_creatures(g);
            if hushed {
                g.director.hush = 1.0;
            }
        });
        let w = windows(&r.pcm);
        let w = w[60..].to_vec();
        let bed = median(&w);
        println!("hush {hushed}: bed {:.1} dB, loudest {:.1} dB", db(bed), db(w.iter().cloned().fold(0.0, f64::max)));
        save(&format!("hush_{}_77_L5", if hushed { "on" } else { "off" }), &r.pcm, &[]);
        out.push((bed, w));
    }
    // events measured against the ordinary room's bed in both runs
    let reference = out[0].0;
    let (bed0, ev0) = (out[0].0, event_energy(&out[0].1, reference));
    let (bed1, ev1) = (out[1].0, event_energy(&out[1].1, reference));
    println!("events: {ev0:.5}/s ordinarily, {ev1:.5}/s hushed");
    assert!(bed1 < bed0, "the room tone dips");
    assert!(ev1 < ev0 * 0.5, "the events thin out and soften: {ev1:.5} vs {ev0:.5}");
}

/// The way down is heard: standing at a walk of 2, 5 and 9 tiles from the
/// exit (by the floor plan), the door's call (`exit-call`: a draught and the
/// sign's relay) rises over the bed when it comes, clearly up close and
/// still from across a few rooms. No compass: this, the door's light and her
/// trail are how the door is found.
///
/// The level is played twice in lockstep (the native stream seeded, each run
/// on a thread of its own), once with the door silent; the difference of the
/// two recordings is the door alone, so a loud prop that happens to stand
/// near the door does not count as the door being heard.
#[test]
fn the_door_is_heard() {
    let mut rows = vec![];
    for (seed, level) in [(4242u32, 6), (77, 8), (9001, 11)] {
        for steps in [2, 5, 9] {
            let run = move |calls: bool| {
                std::thread::Builder::new()
                    .stack_size(256 << 20)
                    .spawn(move || door_run(seed, level, steps, calls))
                    .unwrap()
                    .join()
                    .unwrap()
            };
            let (Some((with, times, metres)), Some((without, _, _))) = (run(true), run(false)) else { continue };
            let door: Vec<f32> = with.iter().zip(&without).map(|(a, b)| a - b).collect();
            let (w, bed) = (windows(&door), median(&windows(&without)[40..]));
            let rise: Vec<f64> = times
                .iter()
                .map(|at| {
                    let a = (at / 0.05) as usize;
                    w[a.min(w.len() - 1)..(a + 50).min(w.len())].iter().cloned().fold(0.0, f64::max) / bed
                })
                .collect();
            let best = rise.iter().cloned().fold(0.0, f64::max);
            println!(
                "seed {seed} L{level}: {steps} tiles from the door ({metres:.0} m straight): {} calls (at {}), the door alone peaks {:+.1} dB against the bed ({:.1} dBFS)",
                times.len(),
                times.iter().map(|t| format!("{t:.1} s")).collect::<Vec<_>>().join(", "),
                db(best),
                db(best * bed)
            );
            save(&format!("door_{seed}_L{level}_{steps}"), &with, &times);
            rows.push((steps, times.len(), best));
        }
    }
    for (steps, calls, best) in &rows {
        assert!(*calls >= 2, "{steps} tiles from the door: {calls} calls in 30 s");
        let want = if *steps <= 5 { 2.0 } else { 1.0 };
        assert!(*best >= want, "{steps} tiles from the door: the call peaks only {:+.1} dB against the bed", db(*best));
    }
}

/// 30 s standing `steps` tiles (by the floor plan) from the exit of
/// `level`, with the door calling or not: the recording, the call times and
/// the straight distance to the door. None when no tile is that far.
fn door_run(seed: u32, level: i32, steps: i32, calls: bool) -> Option<(Vec<f32>, Vec<f64>, f64)> {
    game::base::rng::seed_native(seed as u64);
    let mut r = start(seed, level);
    // the door against the place, not against a creature: none stands on
    // the floor (one standing near, frozen or not, sounds its drone, and
    // the director's tension with it; when this floor's props changed, a
    // Reader came to stand beside paper 6 m from the spot). One frame first,
    // so the creatures placed as play begins are among them
    play(&mut r, 1.0 / 60.0, hold_creatures);
    let g = &mut r.g;
    for e in &g.entities {
        e.g.remove_from_parent();
    }
    g.entities.clear();
    let e = g.exit_data.as_ref().unwrap().pos;
    let d = g.bfs(g.tx(e.x), g.tz(e.z));
    let (x, z) = (0..g.h).flat_map(|z| (0..g.w).map(move |x| (x, z))).find(|&(x, z)| d[(z * g.w + x) as usize] == steps)?;
    g.player.pos.x = g.wx(x);
    g.player.pos.z = g.wz(z);
    let metres = ((g.player.pos.x - e.x).powi(2) + (g.player.pos.z - e.z).powi(2)).sqrt();
    g.reset_director();
    let mut times = vec![];
    let mut last_call = None;
    let mut t: f64 = 0.0;
    play(&mut r, 30.0, |g| {
        hold_creatures(g);
        if !calls {
            g.exit_call_t = 1e9;
        }
        t += 1.0 / 60.0;
        // a new call by its own time (the log keeps only the last few
        // sounds: counting what is in it misses a call once others follow)
        if let Some(at) = g.ambient_log.iter().rev().find(|e| e.tag == "exit-call").map(|e| e.at) {
            if last_call != Some(at) {
                last_call = Some(at);
                times.push(t);
            }
        }
    });
    Some((r.pcm, times, metres))
}

/// The self-assembly's siren, on the Building's first floor, from where you
/// arrive: it stands well out of the bed (it is how you know to go to a
/// cell), from the loudspeakers on the walls, not from you
/// (`docs/MEGASTRUCTURE.md`, B5).
#[test]
fn the_siren_is_heard() {
    game::base::rng::seed_native(0x5135_0001);
    let mut r = start(4242, 1);
    assert!(!r.g.building_speakers.is_empty(), "the Well has loudspeakers");
    // a stretch of the quiet, then the siren
    play(&mut r, 6.0, |_| {});
    r.g.assembly.t = 0.01;
    let mut sirens = vec![];
    let mut t = 6.0;
    play(&mut r, 24.0, |g| {
        t += 1.0 / 60.0;
        if g.assembly.stage == game::world::assembly::Stage::Siren && sirens.is_empty() {
            sirens.push(t);
        }
    });
    let w = windows(&r.pcm);
    let bed = median(&w[20..110]);
    let a = (sirens[0] / 0.05) as usize;
    let peak = w[a..(a + 140).min(w.len())].iter().cloned().fold(0.0, f64::max);
    println!("the Well: bed {:.1} dB, the siren peaks {:+.1} dB over it", db(bed), db(peak / bed));
    save("siren_4242_L1", &r.pcm, &sirens);
    assert!(peak / bed >= 4.0, "the siren rises 12 dB over the bed: {:+.1} dB", db(peak / bed));
}

/// Onsets between `from` and `to` seconds: 10 ms windows over `floor` and
/// over 2.5 times the loudest of the four before the one before (an attack
/// may take two windows), at most one each 80 ms (through the Well's long
/// tail, a rap's attack still stands out of what rings on before it)
fn onsets(pcm: &[f32], from: f64, to: f64, floor: f64) -> Vec<f64> {
    let win = (SR * 0.01) as usize;
    let (a, b) = ((from * SR) as usize, ((to * SR) as usize).min(pcm.len() / 2));
    let rms: Vec<f64> = (a..b)
        .step_by(win)
        .map(|s| {
            let n = win.min(b - s);
            ((0..n).map(|i| ((pcm[2 * (s + i)] + pcm[2 * (s + i) + 1]) as f64 * 0.5).powi(2)).sum::<f64>() / n.max(1) as f64).sqrt()
        })
        .collect();
    let mut out: Vec<f64> = vec![];
    for k in 5..rms.len() {
        let before = rms[k - 5..k - 1].iter().cloned().fold(0.0, f64::max);
        let t = from + k as f64 * 0.01;
        if rms[k] > floor && rms[k] > 2.5 * before && out.last().is_none_or(|l| t - l > 0.08) {
            out.push(t);
        }
    }
    out
}

/// The mix through a band-pass from `lo` to `hi` Hz (two Butterworth
/// high-passes at `lo`, two low-passes at `hi`: 24 dB an octave each side),
/// each channel on its own
fn band_passed(pcm: &[f32], lo: f64, hi: f64) -> Vec<f32> {
    // RBJ's biquads, Q 1/sqrt(2)
    let coef = |f: f64, high: bool| {
        let w = 2.0 * std::f64::consts::PI * f / SR;
        let (sn, cs) = w.sin_cos();
        let a = sn / std::f64::consts::SQRT_2;
        let (b0, b1) = if high { ((1.0 + cs) / 2.0, -(1.0 + cs)) } else { ((1.0 - cs) / 2.0, 1.0 - cs) };
        let a0 = 1.0 + a;
        [b0 / a0, b1 / a0, b0 / a0, -2.0 * cs / a0, (1.0 - a) / a0]
    };
    let stages = [coef(lo, true), coef(lo, true), coef(hi, false), coef(hi, false)];
    let mut out = pcm.to_vec();
    for ch in 0..2 {
        for c in &stages {
            let (mut x1, mut x2, mut y1, mut y2) = (0.0, 0.0, 0.0, 0.0);
            for i in (ch..out.len()).step_by(2) {
                let x = out[i] as f64;
                let y = c[0] * x + c[1] * x1 + c[2] * x2 - c[3] * y1 - c[4] * y2;
                (x2, x1, y2, y1) = (x1, x, y1, y);
                out[i] = y as f32;
            }
        }
    }
    out
}

/// Where a knock is heard beside a lamp: over the ballast's hum (its 60 Hz,
/// 120 Hz and the series up to 1.08 kHz, which fill the whole mix's bed
/// within a few metres of a lamp) and under the grain's hiss
const KNOCK_BAND: (f64, f64) = (1200.0, 6000.0);

/// What was heard of a knock at an answering door and then at a red one,
/// each against the bed at that door (dB over it), over the whole mix and in
/// `KNOCK_BAND`; times in seconds from each knock
struct Knocked {
    /// each door's distance to the nearest lit lamp, metres
    lamp: (f64, f64),
    /// the beds at the answering door and at the red door, dB (the whole
    /// mix; in the band)
    bed: (f64, f64),
    bed_band: (f64, f64),
    /// your raps at the answering door, and your rhythm's gaps
    raps: Vec<f64>,
    raps_band: Vec<f64>,
    mine: Vec<f64>,
    /// someone at the door (from 2 s after the knock until the thing comes
    /// out): its loudest, and what is heard come in the band (its steps and
    /// the hatch's catch), from when it came
    at_door: f64,
    at_door_band: f64,
    came: Vec<f64>,
    /// the thing pushed out, its loudest in the 1.2 s after it began
    answer: f64,
    answer_band: f64,
    out_after: f64,
    /// at the red door: your raps, its raps back, your rhythm
    ours: Vec<f64>,
    ours_band: Vec<f64>,
    back: Vec<f64>,
    back_band: Vec<f64>,
    back_peak_band: f64,
    theirs: Vec<f64>,
    siren: Option<f64>,
}

impl Knocked {
    fn print(&self, what: &str) {
        let r = |v: &[f64]| v.iter().map(|x| (x * 100.0).round() / 100.0).collect::<Vec<_>>();
        println!(
            "{what}: the answering door {:.1} m from a lit lamp, the red door {:.1} m; at {}-{} Hz:",
            self.lamp.0, self.lamp.1, KNOCK_BAND.0, KNOCK_BAND.1
        );
        println!(
            "  answering door: bed {:.1} dB ({:.1} in the band); your raps at {:?} s ({:?} in the band; gaps {:?}); someone at the door {:+.1} dB over the bed ({:+.1} in the band; heard come at {:?} s after it came), the thing pushed out {:+.1} dB ({:+.1} in the band), {:.2} s after the knock",
            self.bed.0,
            self.bed_band.0,
            r(&self.raps),
            r(&self.raps_band),
            r(&self.mine),
            self.at_door,
            self.at_door_band,
            r(&self.came),
            self.answer,
            self.answer_band,
            self.out_after
        );
        let gaps = |o: &[f64]| r(&o.windows(2).map(|p| p[1] - p[0]).collect::<Vec<f64>>());
        println!(
            "  red door: bed {:.1} dB ({:.1} in the band); your raps {:?} ({:?} in the band), its raps back {:?} ({:?} in the band, the loudest {:+.1} dB; gaps {:?} against yours {:?}); the siren {:.1} s after the knock",
            self.bed.1,
            self.bed_band.1,
            r(&self.ours),
            r(&self.ours_band),
            r(&self.back),
            r(&self.back_band),
            self.back_peak_band,
            gaps(&self.back_band),
            r(&self.theirs),
            self.siren.unwrap_or(f64::NAN)
        );
    }
}

/// How far door `d` is from the lamp whose hum is heard where you stand to
/// knock at it (`Game::hum_lamp`: the nearest lit lamp of that storey)
fn door_lamp(g: &Game, d: &game::world::knock::KnockDoor) -> f64 {
    let s = d.stand();
    g.hum_lamp(s.x, s.z).map_or(f64::MAX, |(l, _)| (g.panels[l].wx - d.foot.x).hypot(g.panels[l].wz - d.foot.z))
}

/// Knock at an answering door and then at a red one of `seed`'s Well (the
/// doors `pick` chooses), from where you stand to knock at each; recorded as
/// `target/listen/{name}.wav`
fn knock_and_listen(seed: u32, name: &str, pick: impl Fn(&Game) -> (usize, usize)) -> Knocked {
    use game::world::knock::{ANSWER_AFTER, BACK_AFTER, OUT_AFTER, RAP};
    game::base::rng::seed_native(0x4B0C_0001);
    let mut r = start(seed, 1);
    let (answers, red) = pick(&r.g);
    let lamp = |g: &Game, i: usize| door_lamp(g, &g.knocks.doors[i]);
    let lamps = (lamp(&r.g, answers), lamp(&r.g, red));
    let face = |g: &mut Game, i: usize| {
        let d = g.knocks.doors[i].clone();
        let p = d.stand();
        g.player.pos = engine::math::Vec3::new(p.x, g.floor_y_at(p.x, p.z) + game::prelude::EYE, p.z);
        g.player.yaw = d.normal.0.atan2(d.normal.1);
        g.player.pitch = 0.0;
    };
    r.g.assembly.t = 300.0;
    // the bed at the answering door, then the knock
    face(&mut r.g, answers);
    play(&mut r, 4.0, |_| {});
    let t0 = r.pcm.len() as f64 / 2.0 / SR;
    r.g.knock_at(answers);
    let mine = r.g.knocks.rhythm.clone();
    let mut out_at = None;
    let mut t = t0;
    play(&mut r, 8.0, |g| {
        t += 1.0 / 60.0;
        if out_at.is_none() && g.knocks.given.iter().any(|gv| gv.t >= 0.0) {
            out_at = Some(t);
        }
    });
    // (seen a frame after the frame that brought it out, whose sounds began
    // as that frame began)
    let out_at = out_at.expect("the door answered") - 2.0 / 60.0;
    // the red door: its bed, then the knock
    face(&mut r.g, red);
    play(&mut r, 5.0, |_| {});
    let t1 = r.pcm.len() as f64 / 2.0 / SR;
    r.g.knock_at(red);
    let theirs = r.g.knocks.rhythm.clone();
    let mut siren = None;
    let mut t = t1;
    play(&mut r, 12.0, |g| {
        t += 1.0 / 60.0;
        if siren.is_none() && g.assembly.stage == game::world::assembly::Stage::Siren {
            siren = Some(t);
        }
    });
    save(name, &r.pcm, &[t0, out_at, t1, siren.unwrap_or(t1)]);
    let band = band_passed(&r.pcm, KNOCK_BAND.0, KNOCK_BAND.1);
    let (w, wb) = (windows(&r.pcm), windows(&band));
    let k = |s: f64| ((s / 0.05) as usize).min(w.len());
    // each door's bed: the median of its last three seconds before the knock
    let bed = |w: &[f64], at: f64| median(&w[k(at - 3.0)..k(at)]);
    let peak = |w: &[f64], a: f64, b: f64| w[k(a)..k(b)].iter().cloned().fold(0.0, f64::max);
    let (b0, b1, bb0, bb1) = (bed(&w, t0), bed(&w, t1), bed(&wb, t0), bed(&wb, t1));
    let from = |o: Vec<f64>, at: f64| o.into_iter().map(|x| x - at).collect::<Vec<f64>>();
    // (the steps come `OUT_AFTER` before the thing; the catch at 0.75 s)
    let came_at = out_at - OUT_AFTER;
    // (your raps end within `2 RAP.1` of the knock, before it can knock back)
    let ours_to = t1 + 2.0 * RAP.1 + 0.1;
    let back_from = t1 + BACK_AFTER.0 - 0.05;
    let back_to = t1 + BACK_AFTER.1 + 1.4;
    Knocked {
        lamp: lamps,
        bed: (db(b0), db(b1)),
        bed_band: (db(bb0), db(bb1)),
        raps: from(onsets(&r.pcm, t0 - 0.1, t0 + 1.4, b0 * 4.0), t0),
        raps_band: from(onsets(&band, t0 - 0.1, t0 + 1.4, bb0 * 4.0), t0),
        mine,
        at_door: db(peak(&w, t0 + ANSWER_AFTER.0, out_at) / b0),
        at_door_band: db(peak(&wb, t0 + ANSWER_AFTER.0, out_at) / bb0),
        came: from(onsets(&band, came_at - 0.1, out_at - 0.05, bb0 * 4.0), came_at),
        answer: db(peak(&w, out_at, out_at + 1.2) / b0),
        answer_band: db(peak(&wb, out_at, out_at + 1.2) / bb0),
        out_after: out_at - t0,
        ours: from(onsets(&r.pcm, t1 - 0.1, ours_to, b1 * 4.0), t1),
        ours_band: from(onsets(&band, t1 - 0.1, ours_to, bb1 * 4.0), t1),
        back: from(onsets(&r.pcm, back_from, back_to, b1 * 4.0), t1),
        back_band: from(onsets(&band, back_from, back_to, bb1 * 4.0), t1),
        back_peak_band: db(peak(&wb, back_from, back_to) / bb1),
        theirs,
        siren: siren.map(|s| s - t1),
    }
}

/// A knock at the Well's sealed doors (`docs/MEGASTRUCTURE.md`, G5;
/// `world::knock`), from where you stand to knock: your three raps; at an
/// ordinary door that answers, someone at it and the thing pushed out
/// (heard over the bed seconds after); at a red door, three raps back in
/// your own rhythm, and the siren within seconds. Here at doors 8 m or more
/// from a lamp, over the whole mix, each door against its own bed; beside a
/// lamp, where the ballast's hum fills the bed, `a_knock_is_heard_beside_a_lamp`.
#[test]
fn a_knock_is_answered_and_a_wrong_door_knocks_back() {
    use game::world::knock::{Gift, KnockDoor, Mark, ANSWER_AFTER, OUT_AFTER};
    let quiet = |g: &Game, d: &KnockDoor| g.panels.iter().all(|p| (p.wx - d.foot.x).hypot(p.wz - d.foot.z) >= 8.0);
    let k = knock_and_listen(4242, "knock_4242_L1", |g| {
        let answers = g.knocks.doors.iter().position(|d| d.gift == Some(Gift::Water) && quiet(g, d)).expect("a door that gives water");
        let red = g.knocks.doors.iter().position(|d| d.mark == Mark::Red && quiet(g, d)).expect("a red door");
        (answers, red)
    });
    k.print("the Well, away from the lamps");
    assert_eq!(k.raps.len(), 3, "three raps of your own: {:?}", k.raps);
    assert!(k.at_door >= 6.0, "someone at the door is heard over the bed: {:+.1} dB", k.at_door);
    assert!(k.answer >= 6.0, "the thing pushed out is heard over the bed: {:+.1} dB", k.answer);
    assert!(k.out_after <= ANSWER_AFTER.1 + OUT_AFTER + 0.1);
    assert_eq!(k.ours.len(), 3, "three raps at the red door: {:?}", k.ours);
    assert_eq!(k.back.len(), 3, "three raps back: {:?}", k.back);
    let gaps: Vec<f64> = k.back.windows(2).map(|p| p[1] - p[0]).collect();
    for (g, want) in gaps.iter().zip(&k.theirs) {
        assert!((g - want).abs() < 0.04, "in your own rhythm: {gaps:?} against {:?}", k.theirs);
    }
    assert!(k.siren.is_some_and(|s| s < 9.0), "the siren within seconds of the red door");
}

/// The knock where its doors are nearest a lit lamp (`MILESTONES.md` 12).
/// The ballast's hum sits at the nearest lamp, at full strength within
/// 5.6 m, and there it fills the whole mix's bed with its 120 Hz series
/// (−35 dB against −43 dB away from the lamps, nearly all of it under
/// 1.2 kHz); the knock and someone at the door had been made in that same
/// band (thuds lowpassed at 260–380 Hz, sines at 80–120 Hz) and were not
/// heard there: your three raps never three (none at all at the answering
/// doors), the red door's raps back 1–5 dB over the bed above 1.2 kHz and not
/// one heard as a rap, someone at the door not a step (on 4242 the catch
/// alone). Each is now heard where it carries over the hum, in `KNOCK_BAND`:
/// your three raps and the three raps back in your rhythm, someone at the
/// door (its two steps and the hatch's catch, each heard come: 12 dB over
/// the band's bed in 10 ms) and the thing pushed out, 6 dB or more over it.
/// Over the whole mix the hum's lines still outweigh them (2–5 dB): that is
/// not where they are heard. On the Well of seed 4242 (an answering door
/// 3.9 m from a cell's lamp, a red door 3.1 m), 77 and 1999 (both doors
/// under the lamp of the cell beside them, 0.4 m; on 1999 a note slid under
/// the door). With the answer's sounds at gain 0, this fails.
#[test]
fn a_knock_is_heard_beside_a_lamp() {
    use game::world::knock::Mark;
    // (every floor heard before any is judged)
    let heard: Vec<(u32, Knocked)> = [4242u32, 77, 1999]
        .into_iter()
        .map(|seed| {
            let k = knock_and_listen(seed, &format!("knock_lamp_{seed}_L1"), |g| {
                // the answering door and the red door nearest a lit lamp
                let near = |want: &dyn Fn(&game::world::knock::KnockDoor) -> bool| {
                    (0..g.knocks.doors.len())
                        .filter(|&i| want(&g.knocks.doors[i]))
                        .map(|i| {
                            let d = &g.knocks.doors[i];
                            let m = door_lamp(g, d);
                            (m, i)
                        })
                        .min_by(|a, b| a.0.total_cmp(&b.0))
                        .map(|x| x.1)
                };
                let answers = near(&|d| d.gift.is_some()).expect("a door that answers");
                let red = near(&|d| d.mark == Mark::Red).expect("a red door");
                (answers, red)
            });
            k.print(&format!("seed {seed}'s Well, beside a lamp"));
            (seed, k)
        })
        .collect();
    for (seed, k) in &heard {
        assert!(k.lamp.0 <= 4.0 && k.lamp.1 <= 4.0, "seed {seed}: doors beside a lamp: {:?} m", k.lamp);
        assert_eq!(k.raps_band.len(), 3, "seed {seed}: three raps of your own: {:?}", k.raps_band);
        assert!(k.at_door_band >= 6.0, "seed {seed}: someone at the door is heard over the hum: {:+.1} dB", k.at_door_band);
        assert_eq!(k.came.len(), 3, "seed {seed}: someone heard come to the door, two steps and the catch: {:?}", k.came);
        assert!(k.answer_band >= 6.0, "seed {seed}: the thing pushed out is heard over the hum: {:+.1} dB", k.answer_band);
        assert_eq!(k.ours_band.len(), 3, "seed {seed}: three raps at the red door: {:?}", k.ours_band);
        assert_eq!(k.back_band.len(), 3, "seed {seed}: three raps back: {:?}", k.back_band);
        let gaps: Vec<f64> = k.back_band.windows(2).map(|p| p[1] - p[0]).collect();
        for (g, want) in gaps.iter().zip(&k.theirs) {
            assert!((g - want).abs() < 0.04, "seed {seed}: in your own rhythm: {gaps:?} against {:?}", k.theirs);
        }
        assert!(k.siren.is_some_and(|s| s < 9.0), "seed {seed}: the siren within seconds of the red door");
    }
}

/// What knocks at your cell (`docs/MEGASTRUCTURE.md`, G6;
/// `world::cell_knock`), heard from inside it, sealed, the fog up:
/// its knocks; your answer; then the wheel turned from outside, a pull at a
/// time, and the door giving (its bolts, the air through the gap), each
/// from the door. Measured over the knocking before the answer, in the
/// band the squeal, the bolts' steel and the air fill (1.6–4 kHz: over the
/// knocks' thud, the heart in the dark of the cell and the lamp's hum).
#[test]
fn the_wheel_is_tried_and_the_door_gives() {
    use game::world::assembly::Stage;
    use game::world::cell_knock::{Visit, GIVE};
    game::base::rng::seed_native(0xCE11_0006);
    let mut r = start(4242, 1);
    let camp = r.g.story.camp.as_ref().map(|c| c.tile);
    let cells = r.g.building.as_ref().unwrap().cells.clone();
    let ci = cells.iter().position(|c| Some(c.tile) != camp).expect("a cell");
    let (x, z) = cells[ci].tile;
    let (wx, wz) = (r.g.wx(x), r.g.wz(z));
    let h = r.g.floor_y_of(x, z);
    r.g.player.pos = engine::math::Vec3::new(wx, h + game::prelude::EYE, wz);
    r.g.turn_cell_wheel(ci, true);
    r.g.assembly.t = 1e6;
    play(&mut r, 3.0, |_| {});
    // the fog up, and something at the door (the fog's surface a little
    // under the cell's floor, and its rush held off: when the door gives,
    // what the fog does is not in what is measured)
    {
        let a = &mut r.g.assembly;
        a.stage = Stage::Crest;
        a.y = h - 2.0;
        a.crest = a.y;
        a.t = 60.0;
        a.count = 1;
    }
    let mut knocks = 0;
    // (the fog's own rush held off: what is measured is the door)
    play(&mut r, 6.0, |g| {
        g.assembly.sound_t = 1e9;
        if g.assembly.door.visit == Visit::Knocking {
            knocks = g.assembly.door.knocks;
        }
    });
    assert!(knocks >= 1, "something knocks");
    let t0 = r.pcm.len() as f64 / 2.0 / SR;
    r.g.answer_the_door(ci);
    assert_eq!(r.g.assembly.door.visit, Visit::Trying, "answered");
    let wait = r.g.assembly.door.wait;
    let mut gave = None;
    let mut t = t0;
    play(&mut r, 8.0, |g| {
        g.assembly.sound_t = 1e9;
        t += 1.0 / 60.0;
        if gave.is_none() && g.assembly.door.visit == Visit::Gave {
            gave = Some(t);
        }
    });
    let gave = gave.expect("the door gave");
    let band = windows_band(&r.pcm, 1600.0, 4000.0);
    let at = |s: f64| ((s / 0.05) as usize).min(band.len());
    // the knocking before the answer: its bed (the band's median, and its
    // loudest), and the whole mix's
    let (b0, b1) = (at(3.0), at(t0 - 0.1));
    let bed = median(&band[b0..b1]);
    let bed_top = band[b0..b1].iter().cloned().fold(0.0, f64::max);
    // each pull at the wheel: the band rising 12 dB out of its bed (no knock
    // comes while it tries)
    let tw = &band[at(t0 + wait)..at(gave - 0.05)];
    let pulls: Vec<f64> =
        (0..tw.len()).filter(|&k| tw[k] > bed * 4.0 && (k == 0 || tw[k - 1] <= bed * 4.0)).map(|k| t0 + wait + k as f64 * 0.05).collect();
    let pull_top = tw.iter().cloned().fold(0.0, f64::max);
    let gives = band[at(gave)..at(gave + 1.0)].iter().cloned().fold(0.0, f64::max);
    println!(
        "a cell on the Well, sealed, the fog up: {knocks} knocks, the band's bed {:.1} dB (a knock's ring {:.1}); answered at {t0:.1} s, the wheel from {:.1} s: pulls at {:?} s, the loudest {:+.1} dB over the bed; the door gave {:.2} s after the answer, {:+.1} dB in the band",
        db(bed),
        db(bed_top),
        t0 + wait,
        pulls.iter().map(|p| ((p - t0) * 100.0).round() / 100.0).collect::<Vec<_>>(),
        db(pull_top / bed),
        gave - t0,
        db(gives / bed)
    );
    save("cell_wheel_4242_L1", &r.pcm, &[t0, t0 + wait, gave]);
    let want = (GIVE / 0.8).floor() as usize;
    assert!(pulls.len() >= want, "the wheel heard turning a pull at a time: {} pulls in {GIVE} s ({pulls:?})", pulls.len());
    assert!(gives > bed * 4.0, "the door giving is heard: {:+.1} dB", db(gives / bed));
}

/// What knocks at your cell, from its first knock to its having gone: what
/// was heard of it and when its prompts came and went, in seconds from its
/// going (from when it is first heard going)
struct Going {
    /// the cell's distance to the nearest lit lamp, metres
    lamp: f64,
    /// its knocks
    knocks: u32,
    /// the cell's bed before anything knocked, dB (the whole mix; in the band)
    bed: f64,
    bed_band: f64,
    /// its knocking before it went (unanswered; answered, it is answered at
    /// its first rap): its raps heard come in the band, and its loudest
    /// there over the bed
    knocking: Vec<f64>,
    knocking_band: f64,
    /// answered: anything heard come in the band from after your raps to
    /// its going that is not a pull at the wheel (its raps still to come
    /// were hushed)
    after_answer: Vec<f64>,
    /// the wheel let go (heard come in the band), if it had it
    let_go: Vec<f64>,
    /// its breaths going heard come in the band
    breaths: Vec<f64>,
    /// the loudest of its going over the bed (the whole mix; in the band)
    peak: f64,
    peak_band: f64,
    /// `[E] KNOCK` last shown, its going over (`Visit::Gone`), and the door
    /// to open shown
    knock_until: f64,
    gone: f64,
    open: f64,
    /// the graph's nodes in the cell's bed before anything knocked, and
    /// once its going is over (the fewest over a second of each), and the
    /// knocks' gains still held then
    nodes: (usize, usize),
    held: usize,
    /// held (let go): each pull against your hold, its loudest in the band
    /// over the bed (dB) and whether it is heard come
    strain: Vec<(f64, bool)>,
}

/// Sealed in a cell of seed 4242's Well (the one of the wheel's test),
/// standing in its middle facing the door, the fog up and its rush held
/// off: something knocks, and `how` decides: "unanswered", or "let go"
/// (answered at its first knock, the wheel held from its first pull until
/// it lets go). It goes; recorded as
/// `target/listen/cell_going_{how}_4242_L1.wav` (red marks: the fog up,
/// your answer, its going, its gone)
fn cell_going(how: &str) -> Going {
    use game::world::assembly::Stage;
    use game::world::cell_knock::Visit;
    game::base::rng::seed_native(0xCE11_0013);
    let mut r = start(4242, 1);
    let camp = r.g.story.camp.as_ref().map(|c| c.tile);
    let cells = r.g.building.as_ref().unwrap().cells.clone();
    let ci = cells.iter().position(|c| Some(c.tile) != camp).expect("a cell");
    let (x, z) = cells[ci].tile;
    let (wx, wz) = (r.g.wx(x), r.g.wz(z));
    let h = r.g.floor_y_of(x, z);
    let door = r.g.cell_doors[ci].at;
    r.g.player.pos = engine::math::Vec3::new(wx, h + game::prelude::EYE, wz);
    r.g.player.yaw = (-(door.x - wx)).atan2(-(door.z - wz));
    r.g.player.pitch = 0.0;
    // (the lamp whose hum is heard in the cell: the nearest lit one of its
    // storey, `Game::hum_lamp`; how far it is from the door)
    let lamp = r.g.hum_lamp(wx, wz).map_or(f64::MAX, |(l, _)| (r.g.panels[l].wx - door.x).hypot(r.g.panels[l].wz - door.z));
    r.g.turn_cell_wheel(ci, true);
    r.g.assembly.t = 1e6;
    // (the wheel's own turn, then the cell's bed; the graph's nodes over its
    // last second, the fewest: what stays when nothing is in flight)
    play(&mut r, 4.0, |_| {});
    let mut nodes_before = usize::MAX;
    for _ in 0..60 {
        play(&mut r, 1.0 / 60.0, |_| {});
        nodes_before = nodes_before.min(r.ac.offline_nodes().expect("an offline context"));
    }
    let t_fog = r.pcm.len() as f64 / 2.0 / SR;
    {
        let a = &mut r.g.assembly;
        a.stage = Stage::Crest;
        a.y = h - 2.0;
        a.crest = a.y;
        a.t = 90.0;
        a.count = 1;
    }
    let shown = |g: &Game, text: &str| {
        let p = g.player.pos;
        g.interactables.iter().any(|i| i.prompt == text && i.pos.distance_to_squared(&p) < i.r * i.r && (i.enabled)(g))
    };
    let (mut answered, mut held) = (None, false);
    let (mut going, mut gone, mut open, mut knock_until) = (None, None, None, t_fog);
    let mut pulls: Vec<f64> = vec![];
    // (the pulls against your hold: E kept down since a frame before)
    let mut held_pulls: Vec<f64> = vec![];
    let mut pull_t = 0.0;
    let mut nodes_after = usize::MAX;
    let mut t = t_fog;
    while gone.is_none_or(|g| t < g + 1.5) && t < t_fog + 60.0 {
        play(&mut r, 1.0 / 60.0, |g| {
            g.assembly.sound_t = 1e9;
            // (what the last frame did began as that frame began)
            let before = t - 1.0 / 60.0;
            if g.assembly.door.visit == Visit::Trying && g.assembly.door.pull_t > pull_t {
                pulls.push(before);
                if held {
                    held_pulls.push(before);
                }
            }
            pull_t = g.assembly.door.pull_t;
            if shown(g, "[E] KNOCK") {
                knock_until = t;
            }
            if how == "let go" {
                if answered.is_none() && g.assembly.door.visit == Visit::Knocking {
                    g.answer_the_door(ci);
                    answered = Some(t);
                }
                // (held as a player holds it: a press once it shows, E kept down)
                if answered.is_some() && !held && g.cell_wheel_tried(ci) {
                    g.hold_the_wheel(ci);
                    g.keys.insert("KeyE".into());
                    held = true;
                }
            }
            let visit = g.assembly.door.visit;
            if visit == Visit::Going && going.is_none() {
                going = Some(before);
            }
            if visit == Visit::Gone && gone.is_none() {
                gone = Some(before);
            }
            if going.is_some() && open.is_none() && shown(g, "[E] OPEN THE DOOR") {
                open = Some(before);
            }
            t += 1.0 / 60.0;
        });
        if gone.is_some_and(|g| t > g + 0.5) {
            nodes_after = nodes_after.min(r.ac.offline_nodes().expect("an offline context"));
        }
    }
    r.g.keys.remove("KeyE");
    let (going, gone) = (going.expect("it went"), gone.expect("its going was over"));
    let open = open.expect("the door to open came back");
    let marks: Vec<f64> = [Some(t_fog), answered, Some(going), Some(gone)].into_iter().flatten().collect();
    save(&format!("cell_going_{}_4242_L1", how.replace(' ', "_")), &r.pcm, &marks);
    let band = band_passed(&r.pcm, KNOCK_BAND.0, KNOCK_BAND.1);
    let (w, wb) = (windows(&r.pcm), windows(&band));
    let k = |s: f64| ((s / 0.05) as usize).min(w.len());
    // the cell's bed: sealed, its wheel's turn over, before anything knocked
    let (bed, bed_band) = (median(&w[k(1.5)..k(t_fog)]), median(&wb[k(1.5)..k(t_fog)]));
    let peak = |w: &[f64], a: f64, b: f64| w[k(a)..k(b)].iter().cloned().fold(0.0, f64::max);
    let from = |o: Vec<f64>| o.into_iter().map(|x| x - going).collect::<Vec<f64>>();
    let knocked_to = going - 0.05;
    // (after your three raps, which end within 1.1 s of the answer)
    let after_answer = answered.map_or(vec![], |a| {
        onsets(&band, a + 1.2, going - 0.05, bed_band * 4.0)
            .into_iter()
            .filter(|o| !pulls.iter().any(|p| o - p > -0.05 && o - p < 0.35))
            .collect()
    });
    let heard = onsets(&band, going - 0.05, gone, bed_band * 4.0);
    // each pull against your hold: its loudest in the band over the bed, and
    // whether it is heard come
    let strain: Vec<(f64, bool)> = held_pulls
        .iter()
        .map(|&p| (db(peak(&wb, p, p + 0.45) / bed_band), !onsets(&band, p - 0.05, p + 0.4, bed_band * 4.0).is_empty()))
        .collect();
    let split = if answered.is_some() { 1.min(heard.len()) } else { 0 };
    Going {
        lamp,
        knocks: r.g.assembly.door.knocks,
        bed: db(bed),
        bed_band: db(bed_band),
        knocking: if answered.is_none() { from(onsets(&band, t_fog - 0.05, knocked_to, bed_band * 4.0)) } else { vec![] },
        knocking_band: if answered.is_none() { db(peak(&wb, t_fog, knocked_to) / bed_band) } else { f64::NAN },
        after_answer: from(after_answer),
        let_go: from(heard[..split].to_vec()),
        breaths: from(heard[split..].to_vec()),
        peak: db(peak(&w, going, gone) / bed),
        peak_band: db(peak(&wb, going, gone) / bed_band),
        knock_until: knock_until - going,
        gone: gone - going,
        open: open - going,
        nodes: (nodes_before, nodes_after),
        held: r.g.audio.cell_knocks.len(),
        strain,
    }
}

/// What knocks at your cell is heard, at the door beside the cell's own
/// lamp, as its prompts show (`MILESTONES.md` 13; `world::cell_knock`).
/// Measured where it carries over the lamp's hum, in `KNOCK_BAND`, against
/// the cell's bed before anything knocked (heard come: 12 dB over the
/// band's bed in 10 ms):
/// - its knocking: a rap of every knock heard come, its loudest 12 dB over
///   the bed (the knuckle's click and the leaf's ring of `Soundscape::knock`);
/// - left unanswered, `[E] KNOCK` shown until it is heard going, and its
///   going (not steps: something wet drawn off the door and away), three
///   breaths or more each heard come, the loudest 12 dB over the bed;
/// - answered and held until it lets go: nothing heard knock after your
///   raps (its raps still to come hushed); each of its pulls against your
///   hold heard come, 12 dB over the bed (the spokes' creak of steel under
///   load: the rest of the strain, a 700 Hz creak and the bolts' thud, is
///   under the hum: code review 4b); its hold heard slipping from the
///   wheel, and then its going;
/// - the door to open shown only after the last breath heard, once its
///   going has died away;
/// - once it has gone, nothing of it left in the graph: as many nodes as in
///   the cell's bed before it knocked, no knock's gain held (each held one
///   kept its knock's voice, rendered every quantum for the rest of the
///   run: code review 4).
///
/// With the going's sounds at gain 0 this fails, as with the knock's click
/// and ring at gain 0, or its raps not hushed by your answer, or its
/// knocks' gains kept after its going, or the strain's creak at gain 0
/// (`sys::WHEEL_STRAIN`).
#[test]
fn what_knocks_is_heard_going() {
    use game::world::cell_knock::{GOING_AFTER, GOING_BREATHS};
    for how in ["unanswered", "let go"] {
        let g = cell_going(how);
        let r = |v: &[f64]| v.iter().map(|x| (x * 100.0).round() / 100.0).collect::<Vec<_>>();
        println!(
            "a cell on the Well {:.1} m from a lit lamp, {how}, {} knocks: the bed {:.1} dB ({:.1} in {}-{} Hz); its knocking {:+.1} dB in the band, raps heard come at {:?} s; after your answer {:?}; its pulls against your hold {:?} dB in the band (heard come: {:?}); its going {:+.1} dB over the bed ({:+.1} in the band), the hold slipping at {:?} s, {} of its {GOING_BREATHS} breaths heard come at {:?} s; [E] KNOCK last shown at {:.2} s, gone at {:.2} s, the door to open at {:.2} s; the graph {} nodes before, {} once gone ({} knocks' gains held)",
            g.lamp,
            g.knocks,
            g.bed,
            g.bed_band,
            KNOCK_BAND.0,
            KNOCK_BAND.1,
            g.knocking_band,
            r(&g.knocking),
            r(&g.after_answer),
            g.strain.iter().map(|s| (s.0 * 10.0).round() / 10.0).collect::<Vec<_>>(),
            g.strain.iter().map(|s| s.1).collect::<Vec<_>>(),
            g.peak,
            g.peak_band,
            r(&g.let_go),
            g.breaths.len(),
            r(&g.breaths),
            g.knock_until,
            g.gone,
            g.open,
            g.nodes.0,
            g.nodes.1,
            g.held
        );
        assert!(g.lamp <= 6.0, "{how}: the cell beside its lamp ({:.1} m)", g.lamp);
        assert!(g.breaths.len() >= 3, "{how}: its going heard, breath by breath: {:?}", g.breaths);
        assert!(g.peak_band >= 12.0, "{how}: its going heard over the hum: {:+.1} dB", g.peak_band);
        if how == "let go" {
            assert!(g.after_answer.is_empty(), "{how}: answered, it knocks no more: {:?}", g.after_answer);
            assert!(g.strain.len() >= 3, "{how}: it pulls against your hold: {:?}", g.strain);
            assert!(
                g.strain.iter().all(|&(peak, came)| came && peak >= 12.0),
                "{how}: each pull against your hold heard over the hum: {:?}",
                g.strain
            );
            assert_eq!(g.let_go.len(), 1, "{how}: its hold heard slipping: {:?}", g.let_go);
            assert!(g.breaths[0] >= GOING_AFTER - 0.1, "{how}: its going after it let go: {:?}", g.breaths);
        } else {
            assert!(g.knocking_band >= 12.0, "{how}: its knocking heard over the hum: {:+.1} dB", g.knocking_band);
            assert!(g.knocking.len() as u32 >= g.knocks, "{how}: a rap of each of its {} knocks heard come: {:?}", g.knocks, g.knocking);
            assert!(g.knock_until >= -0.05, "{how}: its knock to answer until it is heard going ({:.2} s)", g.knock_until);
        }
        let last = g.breaths.last().copied().unwrap_or(f64::INFINITY);
        assert!(g.open > last, "{how}: the door to open no sooner than its last breath heard ({:.2} s, the breath at {last:.2} s)", g.open);
        assert!(g.open >= g.gone, "{how}: the door to open once it has gone ({:.2} s, gone {:.2} s)", g.open, g.gone);
        assert_eq!(g.held, 0, "{how}: once it has gone, no knock's gain held");
        assert!(g.nodes.1 <= g.nodes.0, "{how}: once it has gone, the graph as it was ({} nodes, {} before)", g.nodes.1, g.nodes.0);
    }
}

#[test]
fn the_hound_is_heard() {
    // a hound 9 m ahead, held while the room's bed is measured; then it is
    // told where you are: its bay, then its nails and its panting as it runs
    let mut r = start(4242, 6);
    hold_creatures(&mut r.g);
    let ei = {
        let g = &mut r.g;
        let f = g.player.fwd();
        let (x, z) = (g.player.pos.x + f.x * 9.0, g.player.pos.z + f.z * 9.0);
        let (tx, tz) = (g.tx(x), g.tz(z));
        let ei = g.add_entity("hound", tx, tz, 0);
        g.entities[ei].g.set_pos(x, 0.0, z);
        ei
    };
    play(&mut r, 5.0, |g| g.entities[ei].freeze = 1e9);
    let before = r.pcm.len() / 2;
    let mut told = false;
    play(&mut r, 4.0, |g| {
        g.entities[ei].freeze = 0.0;
        if !told {
            let p = g.player.pos;
            g.creature_alert(ei, p.x, p.z);
            told = true;
        }
    });
    let log: Vec<String> =
        r.g.creature(ei).map(|c| c.brain.log.iter().map(|x| format!("{:.1} {} ({})", x.0, x.2.name(), x.3)).collect()).unwrap_or_default();
    println!("hound log: {log:?}; d {:.1}", r.g.creature(ei).map(|c| c.brain.last.dist).unwrap_or(-1.0));
    let w = windows(&r.pcm);
    let b = before / WIN;
    let calm = median(&w[40..b]);
    let peak = w[b..(b + 40).min(w.len())].iter().cloned().fold(0.0, f64::max);
    let after = event_energy(&w[(b + 30).min(w.len())..], calm);
    println!("hound: bed {:.1} dB, bay {:.1} dB (+{:.1} dB), its run {after:.3} over the bed", db(calm), db(peak), db(peak / calm));
    save("hound_4242_L6", &r.pcm, &[before as f64 / SR]);
    assert!(peak > calm * 4.0, "the bay should stand 12 dB over the quiet before it");
    assert!(after > 0.0, "its nails and its panting are heard as it runs");
}

/// The one behind the glass is heard as glass, at its window, by you
/// standing before it: the pane's tick (a knock and its ring), the creak
/// under its weight, wet skin squeaking on the pane, the long groan of the
/// glass bowing in; and its wet raincoat rustling. Each stands over the
/// room's bed where it falls.
#[test]
fn the_one_behind_the_glass_is_heard_as_glass() {
    let mut r = start(4242, 7);
    hold_creatures(&mut r.g);
    // behind a window of this floor (forced where the floor's coin said no),
    // you before it
    let ei = game::entities::glass::showcase(&mut r.g, 3.0).expect("no window on 4242's level 7");
    play(&mut r, 3.0, |_| {});
    let mut marks = vec![];
    let mut heard = vec![];
    for (what, secs) in [("tick", 1.0), ("creak", 2.0), ("squeak", 2.0), ("groan", 3.5), ("rustle", 2.0)] {
        let from = r.pcm.len() / 2;
        marks.push(from as f64 / SR);
        game::entities::glass::call(&mut r.g, ei, what);
        play(&mut r, secs, |_| {});
        heard.push((what, from, r.pcm.len() / 2));
    }
    save("glass_4242_L7", &r.pcm, &marks);
    // each in its own band, its loudest 50 ms against the room's median there
    let quiet = (marks[0] / 0.05) as usize;
    let mut ups = vec![];
    for (what, from, to) in heard {
        let (lo, hi) = match what {
            "tick" => (900.0, 3200.0),
            "creak" => (150.0, 600.0),
            "squeak" => (3000.0, 4400.0),
            "rustle" => (600.0, 2000.0),
            _ => (60.0, 8000.0),
        };
        let w = windows_band(&r.pcm, lo, hi);
        let bed = median(&w[20..quiet]);
        let peak = w[from / WIN..(to / WIN).min(w.len())].iter().cloned().fold(0.0, f64::max);
        let up = db(peak / bed);
        println!("glass: {what} {up:+.1} dB over the room at {lo:.0}-{hi:.0} Hz");
        ups.push((what, up));
    }
    for (what, up) in ups {
        assert!(up > 6.0, "its {what} is not heard over the room (6 dB)");
    }
}

/// The one in the corner hums, low, its back to you 7 m ahead: in its band
/// the hum stands 3 dB over the room, against the same room once it has
/// stopped, so its stopping is heard. (That it stops when it is about to
/// turn is its brain's: `creatures_rules`.)
#[test]
fn the_one_in_the_corner_hums() {
    // the per-frame stream seeded: its notes wander on it (unseeded, the
    // test heard another hum with the clock and with the tests before it);
    // `LISTEN_NATIVE` shifts it, to see the spread
    let shift: u64 = std::env::var("LISTEN_NATIVE").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
    game::base::rng::seed_native(0xC04E + shift * 1_000_003);
    let mut r = start(4242, 6);
    hold_creatures(&mut r.g);
    let ei = {
        let g = &mut r.g;
        let f = g.player.fwd();
        let (x, z) = (g.player.pos.x + f.x * 7.0, g.player.pos.z + f.z * 7.0);
        let (tx, tz) = (g.tx(x), g.tz(z));
        let ei = g.add_entity("corner", tx, tz, 0);
        g.entities[ei].g.set_pos(x, 0.0, z);
        // its back to you
        g.entities[ei].yaw = g.player.yaw + std::f64::consts::PI;
        let c = g.creature_mut(ei).unwrap();
        c.ease.insert("humming", 1.0);
        c.ease.insert("turn_in", 1e9);
        ei
    };
    play(&mut r, 3.0, |g| g.entities[ei].freeze = 1e9);
    let hum_from = r.pcm.len() / 2;
    play(&mut r, 8.0, |g| g.entities[ei].freeze = 0.0);
    // then it is still (frozen: no voice), and the hum stops
    let hum_to = r.pcm.len() / 2;
    play(&mut r, 4.0, |g| g.entities[ei].freeze = 1e9);
    // (the room as it is once the hum stops, in the hum's band, a man's
    // voice and its overtones: against the level's first seconds over the
    // whole mix, it had passed on the director's heartbeat while its rest
    // read to the director as a search, and the hum itself, a soft
    // triangle under the lamps' 120 Hz, added nothing; E10)
    // (its band, a critical band about its notes: a tone is heard over the
    // room's own lines there when it adds as much as they hold: its notes
    // against the room's loudest there, each the loudest tenth of its half
    // seconds, since it breathes between some, now and then three times
    // running)
    let humming = quantile(&band_power(&r.pcm, hum_from + SR as usize, hum_to, 150.0, 225.0), 0.9);
    let stopped = quantile(&band_power(&r.pcm, hum_to + SR as usize / 2, r.pcm.len() / 2, 150.0, 225.0), 0.9);
    let up = 10.0 * (humming / stopped).log10();
    println!("corner: at 150-225 Hz the room is {up:+.1} dB while it hums, against once it stops");
    save("corner_4242_L6", &r.pcm, &[hum_from as f64 / SR, hum_to as f64 / SR]);
    assert!(up > 3.0, "its hum should stand 3 dB over the room in its band, and be heard to stop");
    assert_eq!(r.g.creature(ei).map(|c| c.brain.mode), Some(game::entities::brain::Mode::Hold), "it left its corner");
}

/// The Reader whispers as it reads: fast and without a voice, a room's
/// width off, over the room's bed; it does not stop while it does not see
/// you (its back to you, you still).
#[test]
fn the_reader_whispers() {
    let mut r = start(4242, 6);
    hold_creatures(&mut r.g);
    let ei = {
        let g = &mut r.g;
        let f = g.player.fwd();
        let (x, z) = (g.player.pos.x + f.x * 5.0, g.player.pos.z + f.z * 5.0);
        let (tx, tz) = (g.tx(x), g.tz(z));
        let ei = g.add_entity("reader", tx, tz, 0);
        g.entities[ei].g.set_pos(x, 0.0, z);
        // its back to you, reading where it stands
        g.entities[ei].yaw = g.player.yaw + std::f64::consts::PI;
        let now = g.perf_t;
        let c = g.creature_mut(ei).unwrap();
        c.ease.insert("idle_for", 1e9);
        c.brain.set(game::entities::brain::Mode::Idle, "the test has it read", now);
        ei
    };
    play(&mut r, 4.0, |g| g.entities[ei].freeze = 1e9);
    let bed_end = r.pcm.len() / 2;
    play(&mut r, 8.0, |g| g.entities[ei].freeze = 0.0);
    // (it comes in runs of syllables: over the room's high band most of the
    // time it reads, not every moment)
    let w = windows_band(&r.pcm, 2600.0, 4800.0);
    let b = bed_end / WIN;
    let bed = median(&w[40..b]);
    let whisper = quantile(&w[b + 20..], 0.75);
    println!("reader: bed at 2.6–4.8 kHz {:.1} dB, whispering {:.1} dB (+{:.1})", db(bed), db(whisper), db(whisper / bed));
    save("reader_4242_L6", &r.pcm, &[bed_end as f64 / SR]);
    assert!(whisper > bed * 2.0, "its whisper should stand 6 dB over the room at 2.6–4.8 kHz");
    assert_eq!(r.g.creature(ei).map(|c| c.brain.mode), Some(game::entities::brain::Mode::Idle), "it stopped reading");
}

/// The vending machine is heard (`docs/VENDING.md`, V4): standing 1.5 m
/// before a lit one, its hum (its ballast's 120 Hz series, its buzz and its
/// fan) rises over the room, and stops when it goes dark and when a coin
/// buys its silence; its coin ringing down into the cup and a shove's bang
/// stand out of the bed. With the hum's gain at 0, this fails.
#[test]
fn the_vending_machine_is_heard() {
    use game::core_props::vending::model::D;
    use game::core_props::vending::rules::Verb;
    game::base::rng::seed_native(0x7E2D_0001);
    // a floor with a lit machine
    let found = [(4242u32, 9), (4242, 6), (4242, 5), (4242, 4), (77, 4), (77, 6)].into_iter().find_map(|(seed, level)| {
        let r = start(seed, level);
        let i = r.g.vending.iter().position(|m| m.powered)?;
        Some((r, i, seed, level))
    });
    let (mut r, i, seed, level) = found.expect("a floor with a lit vending machine");
    for e in &r.g.entities {
        e.g.remove_from_parent();
    }
    r.g.entities.clear();
    // no rhythm while it is measured: lit until it is put out
    r.g.vending[i].state.rhythm = (1e9, 0.0, 0.0);
    let stand = |g: &mut Game| {
        let m = &g.vending[i];
        let p = m.world(0.0, 0.0, D / 2.0 + 1.5);
        g.player.pos.x = p.x;
        g.player.pos.z = p.z;
        g.player.yaw = m.yaw();
    };
    stand(&mut r.g);
    let at = r.pcm.len();
    play(&mut r, 8.0, stand);
    let lit_part = r.pcm[at..].to_vec();
    // dark: its rhythm's dark beat
    r.g.vending[i].state.rhythm = (0.001, 1e9, 0.0);
    play(&mut r, 1.0, stand);
    let at = r.pcm.len();
    play(&mut r, 6.0, stand);
    let dark_part = r.pcm[at..].to_vec();
    let band = |pcm: &[f32]| median(&windows_band(pcm, 220.0, 520.0)[20..]);
    let (lit, dark) = (band(&lit_part), band(&dark_part));
    println!(
        "seed {seed} L{level}, 1.5 m before it: its hum's band {:.1} dB lit, {:.1} dB dark ({:+.1} dB)",
        db(lit),
        db(dark),
        db(lit / dark)
    );
    assert!(lit / dark >= 1.6, "its hum is heard over the room: only {:+.1} dB", db(lit / dark));
    // lit again; the coin rings down; then the bang
    r.g.vending[i].state.rhythm = (1e9, 0.0, 0.0);
    play(&mut r, 1.5, stand);
    let at = r.pcm.len();
    r.g.player.coins = 0;
    r.g.vending_use(i, Verb::CoinReturn);
    play(&mut r, 3.0, stand);
    let ring = windows_band(&r.pcm[at..], 2500.0, 7000.0);
    let (peak, bed) = (ring.iter().cloned().fold(0.0, f64::max), median(&ring));
    println!("  the coin's ring peaks {:+.1} dB over its band's bed", db(peak / bed));
    assert!(peak / bed >= 3.0, "the coin is heard ringing down: {:+.1} dB", db(peak / bed));
    let at = r.pcm.len();
    r.g.vending_use(i, Verb::Rock);
    play(&mut r, 1.5, stand);
    let bang = windows(&r.pcm[at..]);
    let (peak, bed) = (bang.iter().cloned().fold(0.0, f64::max), median(&windows(&lit_part)));
    println!("  a shove's bang peaks {:+.1} dB over the room", db(peak / bed));
    assert!(peak / bed >= 3.0, "the bang stands out: {:+.1} dB", db(peak / bed));
    save(&format!("vending_{seed}_L{level}"), &r.pcm, &[]);
}
