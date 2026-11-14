//! `place_fire_alarm_pulls`
use game::prelude::*;
use placers_kit::place::mount_slid;
use placers_kit::place::wall_candidates;
use placers_kit::place::wall_xz;
use placers_kit::wall_plates::blob;
use placers_kit::wall_plates::part;
use placers_kit::wall_plates::scratches;

/// Cached once per thread: a texture every placement shares (its painter
/// draws on a stream of its own, so it comes out the same whenever it is made).
macro_rules! cached_tex {
    ($w:expr_2021, $h:expr_2021, $paint:expr_2021) => {{
        thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
        T.with(|c| c.get_or_init(|| canvas_tex_opt($w, $h, false, $paint)).clone())
    }};
}

// Fire alarm pull: 4% chance per level. A single-action pull station
// flush on its box in the wall (family `wall_hole_pipe_run_posters_and_hooks`'s, the fire alarms'): FIRE raised
// over the lever's well, PULL DOWN on the lever, the glass rod across it, and
// an addressable station's LED blinking as the panel polls it.
pub(crate) fn place_fire_alarm_pulls(g: &mut Game, _dist: &[i32]) {
    g.pp.alarm_pull_positions = vec![]; // reset per level
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = choice(&candidates);
    // the pull station of family `wall_hole_pipe_run_posters_and_hooks` (the fire alarms'), here flush on a box
    // in the wall: its cover's back (z 0.035 in its parts' frame) on the wall,
    // and the LED of an addressable station (family `what_services_screw_to_walls`)
    let alarm = group();
    let o = group();
    o.position().z = -0.008 - 0.035;
    alarm.add(&o);
    let red = lambert!({ map: &pull_paint_tex(), color: 0xffffff });
    let white = lambert!({ color: 0xc4bfb1 });
    let steel = phong!({ color: 0x6a6964, specular: 0x2c2c2c, shininess: 34 });
    let add = |name: &str, mat: &Mat, x: f64, y: f64, z: f64| {
        let m = mesh(pull_part(name, 0.1, 0.14, 0.03), mat);
        m.position().set(x, y, z);
        o.add(&m);
    };
    add("Real__pullBody", &red, 0.0, 0.0, 0.0);
    add("Real__pullLetters", &white, 0.0, 0.0, 0.0);
    add("Real__pullScrews", &steel, 0.0, 0.0, 0.0);
    add("Real__pullLock", &steel, 0.030, -0.050, 0.035 + 0.030 + 0.0035);
    // the lever up in its well, the glass rod across it in its lugs
    let pivot = group();
    pivot.position().set(0, 0.020, 0.035 + 0.030);
    o.add(&pivot);
    pivot.add(&mesh(pull_part("Real__pullHandle", 0.07, 0.03, 0.016), &red));
    pivot.add(&mesh(pull_part("Real__pullHandleText", 0.05, 0.008, 0.001), &white));
    add("Real__pullRod", &lambert!({ color: 0xc8d6d0, transparent: true, opacity: 0.5 }), 0.0, 0.002, 0.035 + 0.030 + 0.0045);
    // The station's LED, dark between the panel's polls.
    let led_mat = basic_mat!({ color: 0x3a0e0a });
    let led = mesh(pull_part("Real__fireAlarmPullLed", 0.007, 0.007, 0.002), &led_mat);
    led.position().set(0.036, 0.052, 0.035 + 0.030);
    o.add(&led);
    // Its blink: a short red flash every four seconds (the old throb's phase).
    let pulse_phase = rnd() * PI * 2.0;
    g.animate(move |_g, _dt, t| {
        let k = (t * 0.25 + pulse_phase / (PI * 2.0)) % 1.0;
        led_mat.m().color.set_hex(if k < 0.035 { 0xd8402c } else { 0x3a0e0a });
    });
    let (wall_x, wall_z) = wall_xz(g, c, 0.008);
    mount_slid(&alarm, c, wall_x, wall_z, 1.50, 0.6);
    g.world.add(&alarm);
    // Track for beep audio.
    let p = alarm.pos();
    g.pp.alarm_pull_positions.push(v3(p.x, 1.50, p.z));
}

fn pull_part(name: &str, w: f64, h: f64, d: f64) -> Geo {
    part(name, w, h, d)
}

/// The pull station's cover: red enamel over die-cast, rubbed to the metal at
/// its edges and round the lever's well where hands go, grimy, a few chips
/// (as family `wall_hole_pipe_run_posters_and_hooks`'s fire alarms; laid on its front by the part's box
/// projection: the whole texture is the face).
fn pull_paint_tex() -> Tex {
    cached_tex!(128, 170, |ctx, w, h| {
        ctx.set_fill_style("#8a1c14");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..400 {
            ctx.set_fill_style(format!("rgba({},0,0,{})", if rnd() < 0.5 { 40 } else { 200 }, 0.03 + rnd() * 0.05));
            ctx.fill_rect(rnd() * w, rnd() * h, 1.0 + rnd() * 3.0, 1.0 + rnd() * 3.0);
        }
        ctx.set_stroke_style("rgba(150,120,110,0.35)");
        ctx.set_line_width(3);
        ctx.stroke_rect(1.5, 1.5, w - 3.0, h - 3.0);
        for &(x, y) in &[(3.0, 3.0), (w - 4.0, 3.0), (3.0, h - 4.0), (w - 4.0, h - 4.0), (w * 0.5, h - 3.0), (3.0, h * 0.4)] {
            ctx.set_fill_style("rgba(120,116,110,0.85)");
            ctx.begin_path();
            ctx.ellipse(x, y, 2.0 + rnd() * 4.0, 1.5 + rnd() * 3.0, rnd() * 3.0, 0, 7);
            ctx.fill();
        }
        blob(ctx, w * 0.5, h * 0.55, w * 0.6, 1.0, (30, 20, 16, 0.30));
        let gd = ctx.create_linear_gradient(0, 0, 0, h);
        gd.add_color_stop(0, "rgba(160,150,130,0.22)");
        gd.add_color_stop(0.08, "rgba(160,150,130,0)");
        gd.add_color_stop(0.8, "rgba(40,30,24,0)");
        gd.add_color_stop(1, "rgba(40,30,24,0.25)");
        ctx.set_fill_style(gd);
        ctx.fill_rect(0, 0, w, h);
        scratches(ctx, w, h, 14, (170, 140, 130), 0.2, 16.0);
    })
}
