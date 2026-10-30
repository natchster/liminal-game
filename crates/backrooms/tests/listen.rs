//! Listening, headless: levels are played with the audio graph rendered
//! offline, recorded to WAV under `target/listen/` (to listen to) with a
//! loudness timeline next to each (PNG), and measured in 50 ms windows:
//!
//! * a level opens quietly: nothing piles up in its first second;
//! * the moment a creature notices you is loud against what came before;
//! * the director's hush quiets the ambience: fewer and softer events, a
//!   lower room tone.
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
/// standing before it: the pane's tick, the creak under its weight, wet
/// skin squeaking on the pane, the long groan of the glass bowing in; and
/// inside, the water dripping off its coat. Each stands over the room's
/// bed where it falls.
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
    for (what, secs) in [("tick", 1.0), ("creak", 2.0), ("squeak", 2.0), ("groan", 3.5), ("drip", 2.0)] {
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
            "tick" => (2500.0, 6000.0),
            "creak" => (150.0, 600.0),
            "squeak" => (2000.0, 3400.0),
            "drip" => (500.0, 3500.0),
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
