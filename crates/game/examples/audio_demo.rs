//! Listen to the game's audio through the default output device.
//!
//!   cargo run -p game --example audio_demo            # a ~25 s tour
//!   cargo run -p game --example audio_demo all        # everything (~2 min)
//!   cargo run -p game --example audio_demo list       # the item labels
//!   cargo run -p game --example audio_demo child hum  # labels containing any of the words
//!
//! The items cover the sfx table, the persistent beds and the soundscape. `AUDIO_DEMO_MUTE=1`
//! runs the same sequence silently (a smoke run); it prints the audio clock
//! against the wall clock at the end (a lagging clock means the render
//! thread could not keep up).

use game::audio::soundscape::PHRASES;
use game::audio::sys::SFX_NAMES;
use game::Game;
use std::time::{Duration, Instant};

type Item = (String, Box<dyn Fn(&mut Game)>);

fn wait(s: f64) {
    std::thread::sleep(Duration::from_secs_f64(s));
}
/// run `f(g, dt, t)` at 60 Hz for `secs` (the game's update loop)
fn run(g: &mut Game, secs: f64, mut f: impl FnMut(&mut Game, f64, f64)) {
    let t0 = Instant::now();
    let dt = 1.0 / 60.0;
    loop {
        let t = t0.elapsed().as_secs_f64();
        if t >= secs {
            break;
        }
        f(g, dt, t);
        wait(dt);
    }
}
/// fade one of the persistent beds up, hold, and back down
fn bed(g: &mut Game, pick: fn(&Game) -> engine::audio::AudioNode, level: f64, secs: f64) {
    let n = pick(g);
    run(g, secs, |_, dt, t| {
        let target = if t < secs - 0.6 { level } else { 0.0 };
        let gp = n.gain();
        gp.set_value(gp.value() + (target - gp.value()) * (dt * 3.0f64).min(1.0));
    });
    n.gain().set_value(0.0);
}

fn items() -> Vec<Item> {
    let mut v: Vec<Item> = vec![];
    let long = |n: &str| match n {
        "glitch" | "chime" | "guideChime" | "piano" => 2.0,
        "distantScream" | "scream" | "cry" => 1.6,
        "distantFootsteps" | "laugh" => 1.4,
        _ => 0.9,
    };
    for &n in SFX_NAMES.iter() {
        let d = long(n);
        v.push((
            format!("sfx {n}"),
            Box::new(move |g: &mut Game| {
                g.sfx_by_name(n);
                wait(d);
            }),
        ));
    }
    v.push((
        "sfx step walk+run".into(),
        Box::new(|g: &mut Game| {
            for i in 0..8 {
                g.sfx_step(i >= 4);
                wait(if i >= 4 { 0.3 } else { 0.45 });
            }
        }),
    ));
    for surf in ["wetcarpet", "carpet", "office", "concrete", "tile", "stair"] {
        let zone = match surf {
            "tile" => "pool",
            "concrete" => "hall",
            "stair" => "stairwell",
            _ => "lobby",
        };
        v.push((
            format!("footstep {surf} ({zone} reverb)"),
            Box::new(move |g: &mut Game| {
                let s = g.soundscape().unwrap();
                s.set_zone(zone, 0.001);
                for i in 0..6 {
                    s.footstep(surf, if i < 3 { 0.62 } else { 1.0 }, None);
                    wait(0.42);
                }
                wait(0.8);
            }),
        ));
    }
    v.push((
        "hum (fixture overhead, then a swell)".into(),
        Box::new(|g: &mut Game| {
            let s = g.soundscape().unwrap();
            s.set_zone("lobby", 0.001);
            run(g, 4.0, |_, dt, t| {
                s.set_listener(0.0, 1.6, 0.0, 0.0, 0.0, -1.0);
                s.hum.update(dt, 0.0, 3.0, -1.0, if t < 2.0 { 1.0 } else { 2.6 });
            });
            s.hum.silence();
            wait(0.5);
        }),
    ));
    v.push((
        "hum detuned (the Loop)".into(),
        Box::new(|g: &mut Game| {
            let s = g.soundscape().unwrap();
            s.hum.detune(-110.0, 1.5);
            run(g, 3.0, |_, dt, _| s.hum.update(dt, 0.0, 3.0, -1.0, 0.8));
            s.hum.silence();
            s.hum.detune(0.0, 0.1);
            wait(0.5);
        }),
    ));
    v.push((
        "screech".into(),
        Box::new(|g: &mut Game| {
            g.soundscape().unwrap().screech(1.0, 1.0);
            wait(1.6);
        }),
    ));
    v.push((
        "heartbeat wrong".into(),
        Box::new(|g: &mut Game| {
            g.soundscape().unwrap().heartbeat(92.0, 5, 1.0, None);
            wait(4.0);
        }),
    ));
    v.push((
        "heartbeat calm".into(),
        Box::new(|g: &mut Game| {
            g.soundscape().unwrap().heartbeat(80.0, 4, 0.0, None);
            wait(3.2);
        }),
    ));
    v.push((
        "knock above".into(),
        Box::new(|g: &mut Game| {
            let s = g.soundscape().unwrap();
            s.set_listener(0.0, 0.0, 0.0, 0.0, 0.0, -1.0);
            s.knock(3, 0.0, 6.0, -3.0, 0.6);
            wait(5.0);
        }),
    ));
    for &(ph, _) in PHRASES.iter() {
        v.push((
            format!("child {ph} (stairwell)"),
            Box::new(move |g: &mut Game| {
                let s = g.soundscape().unwrap();
                s.set_zone("stairwell", 0.001);
                let voice = s.create_child_voice(None);
                voice.set_level(0.9, 0.001);
                let d = voice.say(ph, None, 1.0, 1.0);
                wait(d + 1.0);
                voice.stop();
                wait(0.3);
            }),
        ));
    }
    v.push((
        "entity growl circling (HRTF + occlusion)".into(),
        Box::new(|g: &mut Game| {
            let s = g.soundscape().unwrap();
            s.set_zone("maze", 0.001);
            let slot = s.voice_slot(2.2);
            let ac = g.ac.clone().unwrap();
            let mut next = 0.0;
            run(g, 6.0, |_, _, t| {
                s.set_listener(0.0, 1.6, 0.0, 0.0, 0.0, -1.0);
                slot.at(4.0 * (t * 1.1).sin(), 1.9, -4.0 * (t * 1.1).cos());
                slot.occlude(if t > 3.0 { 1.0 } else { 0.0 });
                if t >= next {
                    next += 2.0;
                    let tt = ac.current_time();
                    s.tone(&slot.input, tt, 2.4, 52.0, 0.14, "sawtooth", Some(44.0), 0.2);
                    s.burst(&slot.input, tt, 2.4, "lowpass", 320.0, 1.5, 0.09, Some(220.0), 0.2);
                }
            });
            wait(0.5);
        }),
    ));
    v.push(("bed drone".into(), Box::new(|g: &mut Game| bed(g, |g| g.audio.drone_gain.clone().unwrap(), 0.35, 3.0))));
    v.push(("bed payphone".into(), Box::new(|g: &mut Game| bed(g, |g| g.audio.phone_gain.clone().unwrap(), 0.14, 2.5))));
    v.push(("bed vacuum".into(), Box::new(|g: &mut Game| bed(g, |g| g.audio.vacuum_gain.clone().unwrap(), 0.12, 2.5))));
    v.push(("bed safe room".into(), Box::new(|g: &mut Game| bed(g, |g| g.audio.safe_room_gain.clone().unwrap(), 0.25, 3.5))));
    v.push(("bed room tone".into(), Box::new(|g: &mut Game| bed(g, |g| g.audio.room_tone_gain.clone().unwrap(), 0.04, 2.5))));
    v.push(("bed rain hiss".into(), Box::new(|g: &mut Game| bed(g, |g| g.audio.rain_hiss_gain.clone().unwrap(), 0.05, 2.5))));
    v.push((
        "bed stamina breath".into(),
        Box::new(|g: &mut Game| {
            let (bg, sg, pan) = (
                g.audio.stamina_breath_gain.clone().unwrap(),
                g.audio.stamina_breath_sub_gain.clone().unwrap(),
                g.audio.stamina_breath_panner.clone().unwrap(),
            );
            let ac = g.ac.clone().unwrap();
            run(g, 4.0, |_, _, t| {
                let now = ac.current_time();
                let mag = if t < 3.4 { 1.0 } else { 0.0 };
                bg.gain().set_target_at_time(0.09 * mag, now, 0.3);
                sg.gain().set_target_at_time(0.02 * mag, now, 0.3);
                pan.pan().set_target_at_time(0.2 * (t * 0.75).sin(), now, 0.3);
            });
            wait(0.4);
        }),
    ));
    v
}

const TOUR: [&str; 14] = [
    "sfx door",
    "sfx pickup",
    "sfx step walk+run",
    "footstep wetcarpet",
    "footstep concrete",
    "footstep stair",
    "hum (fixture",
    "child sob",
    "child help",
    "screech",
    "heartbeat wrong",
    "entity growl",
    "bed drone",
    "sfx glitch",
];

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let all = items();
    if args.iter().any(|a| a == "list") {
        for (l, _) in &all {
            println!("{l}");
        }
        return;
    }
    let chosen: Vec<&Item> = if args.iter().any(|a| a == "all") {
        all.iter().collect()
    } else if args.is_empty() {
        TOUR.iter().filter_map(|p| all.iter().find(|(l, _)| l.starts_with(p))).collect()
    } else {
        all.iter().filter(|(l, _)| args.iter().any(|a| l.contains(a.as_str()))).collect()
    };
    let mut g = Game::new_bare();
    g.audio_init();
    let ac = g.ac.clone().unwrap();
    if std::env::var("AUDIO_DEMO_MUTE").is_ok_and(|v| v != "0") {
        ac.set_muted(true);
    }
    let (wall0, clock0) = (Instant::now(), ac.current_time());
    println!("audio: {} Hz{}", ac.sample_rate(), if chosen.is_empty() { " — nothing matched (try `list`)" } else { "" });
    let s = g.soundscape().unwrap();
    s.set_listener(0.0, 1.6, 0.0, 0.0, 0.0, -1.0);
    s.set_zone("lobby", 0.001);
    wait(0.3);
    for (label, f) in chosen {
        println!("▶ {label}");
        f(&mut g);
    }
    wait(0.5);
    println!("wall {:.2} s, audio clock {:.2} s", wall0.elapsed().as_secs_f64(), ac.current_time() - clock0);
}
