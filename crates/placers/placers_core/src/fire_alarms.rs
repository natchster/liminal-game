//! `place_fire_alarms`
use game::prelude::*;
use placers_kit::place::pick;
use placers_kit::place::wall_candidates;
use placers_kit::wall_plates::part;

// Fire alarm pull-station: 25% chance per level, 1 unit (Blender batch
// `r06`): a die-cast single-action station on its red surface back box, FIRE
// raised over its lever, PULL DOWN on the lever, the glass rod across it, the
// cover's screws, the conduit up the wall to a junction box. Now and then it
// has been pulled: the lever down, the rod snapped, its glass on the floor.
pub(crate) fn place_fire_alarms(g: &mut Game, _dist: &[i32]) {
    g.pp.fire_alarm_positions.clear(); // reset per level
    if rnd() < 0.75 {
        return;
    } // 25% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = pick(&candidates);
    let alarm = group();
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.023);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.023);
    let alarm_height = 1.3 + rnd() * 0.3; // 1.3-1.6 m (accessibility standard)
    if c.dx.abs() == 1 {
        alarm.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        alarm.position().set(wall_x, alarm_height, wall_z + (rnd() - 0.5) * 1.2);
    } else {
        alarm.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        alarm.position().set(wall_x + (rnd() - 0.5) * 1.2, alarm_height, wall_z);
    }
    g.world.add(&alarm);
    // Track for beep audio events.
    let ap = alarm.pos();
    g.pp.fire_alarm_positions.push(v3(ap.x, alarm_height, ap.z));
    let ceil = g.ceil_at_world(g.wx(c.x), g.wz(c.z));
    // 15% chance: it was pulled; the glass rod's pieces on the floor below.
    let pulled = rnd() < 0.15;
    let th = alarm.rot().y;
    let (along, out) = ((th.cos(), -th.sin()), (th.sin(), th.cos()));
    if pulled {
        let glass = lambert!({ color: 0xc4d2cc, transparent: true, opacity: 0.55 });
        let shard_count = 3 + (rnd() * 4.0).floor() as i32;
        for _ in 0..shard_count {
            let shard_w = 0.015 + rnd() * 0.020;
            let shard_h = 0.020 + rnd() * 0.020;
            let tip = (rnd() - 0.5) * 0.4;
            let turn = rnd() * PI * 2.0;
            let ang = rnd() * PI * 2.0;
            let rad = 0.05 + rnd() * 0.25;
            // a piece of the 5 mm rod, lying in front of the wall
            let shard = mesh(part("Real__rodShard", 0.02, 0.005, 0.005), &glass);
            let k = ((shard_w + shard_h) / 0.045).clamp(0.85, 1.15);
            shard.scale().set(k, k, k);
            shard.rotation().set(tip * 0.2, turn, 0);
            // in front of the wall, not in it
            let (a, n) = (ang.cos() * rad, 0.03 + ang.sin().abs() * rad);
            shard.position().set(ap.x + along.0 * a + out.0 * n, 0.0, ap.z + along.1 * a + out.1 * n);
            g.world.add(&shard);
        }
    }
    let st = build_pull_station(pulled, alarm_height, ceil);
    alarm.add(&st);
}

/// The station: its group's origin 23 mm off the wall at the cover's middle
/// (the wall at z = -0.023), the conduit from its back box up to a junction
/// box under the ceiling `ceil` (absolute; the station at `at_y`).
fn build_pull_station(pulled: bool, at_y: f64, ceil: f64) -> Obj {
    let st = group();
    let o = group();
    o.position().z = -0.023;
    st.add(&o);
    let red = lambert!({ map: &alarm_paint_tex(), color: 0xffffff });
    let red_box = lambert!({ color: 0x7c1a13 });
    let white = lambert!({ color: 0xc4bfb1 });
    let steel = phong!({ color: 0x6a6964, specular: 0x2c2c2c, shininess: 34 });
    let zinc = phong!({ color: 0x7c7d79, specular: 0x262626, shininess: 22 });
    let glass = lambert!({ color: 0xc8d6d0, transparent: true, opacity: 0.5 });
    let add = |name: &str, mat: &Mat, x: f64, y: f64, z: f64| {
        let m = mesh(part(name, 0.1, 0.14, 0.03), mat);
        m.position().set(x, y, z);
        o.add(&m);
        m
    };
    add("Real__pullBox", &red_box, 0.0, 0.0, 0.0);
    add("Real__pullBody", &red, 0.0, 0.0, 0.0);
    add("Real__pullLetters", &white, 0.0, 0.0, 0.0);
    add("Real__pullScrews", &steel, 0.0, 0.0, 0.0);
    add("Real__pullLock", &steel, 0.030, -0.050, 0.035 + 0.030 + 0.0035);
    // the lever on its pivot, at the top of its well; down when pulled
    let pivot = group();
    pivot.position().set(0, 0.004 + 0.020 - 0.004, 0.035 + 0.030);
    pivot.rotation().x = if pulled { -0.75 } else { 0.0 };
    o.add(&pivot);
    let lever = mesh(part("Real__pullHandle", 0.07, 0.03, 0.016), &red);
    pivot.add(&lever);
    let text = mesh(part("Real__pullHandleText", 0.06, 0.008, 0.001), &white);
    pivot.add(&text);
    // the glass rod across it in its lugs (snapped when it was pulled)
    let rod = mesh(part(if pulled { "Real__pullRodBroken" } else { "Real__pullRod" }, 0.09, 0.005, 0.005), &glass);
    rod.position().set(0, 0.002, 0.035 + 0.030 + 0.0045);
    o.add(&rod);
    // the conduit: a connector in the box's knockout, EMT up the wall on its
    // straps, into a junction box under the ceiling
    let top = (ceil - 0.10).min(2.85) - at_y; // the junction box's top, above the station
    let jb_c = top - 0.051;
    add("Real__emtConnector", &zinc, 0.0, 0.070, 0.0175);
    let c0 = 0.070 + 0.030;
    let c1 = jb_c - 0.051;
    if c1 > c0 + 0.05 {
        let len = c1 - c0;
        let emt = mesh(cyl_geo!(0.00895, 0.00895, len, 10, 1, true), &zinc);
        emt.position().set(0, (c0 + c1) / 2.0, 0.0175);
        o.add(&emt);
        let n = ((len / 1.0).ceil() as i32).max(1);
        for k in 0..n {
            let y = c0 + (k as f64 + 0.5) * len / n as f64;
            add("Real__emtStrap", &zinc, 0.0, y, 0.0);
        }
        let jb = mesh(box_geo!(0.102, 0.102, 0.054), lambert!({ color: 0x75766f }));
        jb.position().set(0, jb_c, 0.027);
        o.add(&jb);
        let cover = mesh(box_geo!(0.106, 0.106, 0.004), lambert!({ color: 0x6c6d66 }));
        cover.position().set(0, jb_c, 0.056);
        o.add(&cover);
    }
    st
}

/// The station's cover: red enamel over die-cast, rubbed to the metal at its
/// edges and round the lever's well where hands go, grimy, a few chips.
/// (Laid on its front by the part's box projection: the whole texture is the face.)
fn alarm_paint_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            canvas_tex_opt(128, 170, false, |ctx, w, h| {
                ctx.set_fill_style("#8a1c14");
                ctx.fill_rect(0, 0, w, h);
                // the enamel's mottle
                for _ in 0..400 {
                    ctx.set_fill_style(format!("rgba({},0,0,{})", if rnd() < 0.5 { 40 } else { 200 }, 0.03 + rnd() * 0.05));
                    ctx.fill_rect(rnd() * w, rnd() * h, 1.0 + rnd() * 3.0, 1.0 + rnd() * 3.0);
                }
                // rubbed thin at the edges, chipped to grey metal at the corners
                ctx.set_stroke_style("rgba(150,120,110,0.35)");
                ctx.set_line_width(3);
                ctx.stroke_rect(1.5, 1.5, w - 3.0, h - 3.0);
                for &(x, y) in &[(3.0, 3.0), (w - 4.0, 3.0), (3.0, h - 4.0), (w - 4.0, h - 4.0), (w * 0.5, h - 3.0), (w - 3.0, h * 0.6)] {
                    ctx.set_fill_style("rgba(120,116,110,0.85)");
                    ctx.begin_path();
                    ctx.ellipse(x, y, 2.0 + rnd() * 4.0, 1.5 + rnd() * 3.0, rnd() * 3.0, 0, 7);
                    ctx.fill();
                }
                // hands: grime round the lever's well and below it
                let gr = ctx.create_radial_gradient(w * 0.5, h * 0.52, 8, w * 0.5, h * 0.55, w * 0.62);
                gr.add_color_stop(0, "rgba(30,20,16,0.30)");
                gr.add_color_stop(1, "rgba(30,20,16,0)");
                ctx.set_fill_style(gr);
                ctx.fill_rect(0, 0, w, h);
                // dust on its top edge, the grime of years low down
                let gd = ctx.create_linear_gradient(0, 0, 0, h);
                gd.add_color_stop(0, "rgba(160,150,130,0.22)");
                gd.add_color_stop(0.08, "rgba(160,150,130,0)");
                gd.add_color_stop(0.8, "rgba(40,30,24,0)");
                gd.add_color_stop(1, "rgba(40,30,24,0.25)");
                ctx.set_fill_style(gd);
                ctx.fill_rect(0, 0, w, h);
                // scratches
                ctx.set_line_width(0.8);
                for _ in 0..14 {
                    let (x, y, a, l) = (rnd() * w, rnd() * h, rnd() * 6.3, 6.0 + rnd() * 20.0);
                    ctx.set_stroke_style(format!("rgba(170,140,130,{})", 0.2 + rnd() * 0.3));
                    ctx.begin_path();
                    ctx.move_to(x, y);
                    ctx.line_to(x + a.cos() * l, y + a.sin() * l);
                    ctx.stroke();
                }
            })
        })
        .clone()
    })
}
