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
    let mut r = start(4242, 6);
    hold_creatures(&mut r.g);
    // a smiler 9 m ahead with its back to you, then it turns
    let (ei, yaw) = {
        let g = &mut r.g;
        let ei = g.entities.iter().position(|e| e.kind == "smiler").expect("a smiler");
        let f = g.player.fwd();
        let (x, z) = (g.player.pos.x + f.x * 9.0, g.player.pos.z + f.z * 9.0);
        g.entities[ei].g.set_pos(x, 0.0, z);
        (ei, g.player.yaw)
    };
    play(&mut r, 6.0, |g| {
        let e = &mut g.entities[ei];
        e.freeze = 1e9;
        e.yaw = yaw + std::f64::consts::PI;
    });
    let before = r.pcm.len() / 2;
    let mut noticed_at = None;
    play(&mut r, 4.0, |g| {
        let e = &mut g.entities[ei];
        if e.state != "hunt" {
            e.freeze = 0.0;
            e.yaw = yaw;
        } else if noticed_at.is_none() {
            noticed_at = Some(g.perf_t);
        }
    });
    assert!(noticed_at.is_some(), "it noticed you");
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
                "seed {seed} L{level}: {steps} tiles from the door ({metres:.0} m straight): {} calls, the door alone peaks {:+.1} dB against the bed ({:.1} dBFS)",
                times.len(),
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
    let g = &mut r.g;
    let e = g.exit_data.as_ref().unwrap().pos;
    let d = g.bfs(g.tx(e.x), g.tz(e.z));
    let (x, z) = (0..g.h).flat_map(|z| (0..g.w).map(move |x| (x, z))).find(|&(x, z)| d[(z * g.w + x) as usize] == steps)?;
    g.player.pos.x = g.wx(x);
    g.player.pos.z = g.wz(z);
    let metres = ((g.player.pos.x - e.x).powi(2) + (g.player.pos.z - e.z).powi(2)).sqrt();
    g.reset_director();
    let mut times = vec![];
    let mut t = 0.0;
    play(&mut r, 30.0, |g| {
        hold_creatures(g);
        if !calls {
            g.exit_call_t = 1e9;
        }
        t += 1.0 / 60.0;
        if g.ambient_log.iter().filter(|e| e.tag == "exit-call").count() > times.len() {
            times.push(t);
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
