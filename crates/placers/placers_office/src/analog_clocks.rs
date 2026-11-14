//! `place_analog_clocks`
use game::prelude::*;
use placers_kit::place::pick_wall;
use placers_kit::place::wall_mount;

// Analog wall clock: 3% chance per level. Round face + slowly moving hands.
pub(crate) fn place_analog_clocks(g: &mut Game, _dist: &[i32]) {
    g.pp.clock_positions.clear(); // reset per level
    if rnd() < 0.97 {
        return;
    }
    let Some((x, z, dx, dz)) = pick_wall(g) else { return };
    let rim_mat = lambert!({ color: 0x303028 });
    let face_mat = lambert!({ map: clock_face_texture() });
    let hand_mat = lambert!({ color: 0x101010 });
    let clock = group();
    let mount_y = 1.90;
    let rim = mesh(pg("AnalogClocks__rim", cyl_geo!(0.15, 0.15, 0.030, 20)), &rim_mat);
    rim.rotation().x = PI / 2.0;
    rim.position().y = mount_y;
    clock.add(&rim);
    let face = mesh(circle_geo!(0.145, 20), &face_mat);
    face.position().set(0, mount_y, 0.016);
    clock.add(&face);
    // Hour hand (private geometry: pivot moved to its end).
    let hour_hand = mesh(pg_own("AnalogClocks__hourHand", box_geo!(0.008, 0.070, 0.004)), &hand_mat);
    hour_hand.position().set(0, mount_y, 0.019);
    hour_hand.geo().translate(0, 0.035, 0);
    clock.add(&hour_hand);
    // Minute hand (longer, thinner).
    let minute_hand = mesh(pg_own("AnalogClocks__minuteHand", box_geo!(0.005, 0.100, 0.004)), &hand_mat);
    minute_hand.position().set(0, mount_y, 0.021);
    minute_hand.geo().translate(0, 0.050, 0);
    clock.add(&minute_hand);
    // 60% chance: hands still move.
    let is_broken = rnd() < 0.4;
    if !is_broken {
        // minute hand: full revolution per 60 s; hour hand: per 720 s.
        let (mh, hh) = (minute_hand.clone(), hour_hand.clone());
        g.animate(move |_g, _dt, t| {
            mh.rotation().z = -t * (PI * 2.0 / 60.0);
            hh.rotation().z = -t * (PI * 2.0 / 720.0);
        });
    } else {
        // Frozen at random angle.
        minute_hand.rotation().z = rnd() * PI * 2.0;
        hour_hand.rotation().z = rnd() * PI * 2.0;
    }
    wall_mount(g, &clock, x, z, dx, dz, 0.015);
    g.world.add(&clock);
    if !is_broken {
        let p = clock.pos();
        g.pp.clock_positions.push(v3(p.x, 0, p.z)); // only ticking clocks
    }
}

// `clock_face_texture`: 12 tick marks + centre dot.
game::lazy_static_tex!(
    clock_face_texture,
    canvas_tex(160, 160, |ctx, w, h| {
        ctx.set_fill_style("#f0ece0");
        ctx.fill_rect(0, 0, w, h);
        let (cx, cy) = (w / 2.0, h / 2.0);
        // 12 tick marks around edge.
        ctx.set_stroke_style("#202020");
        ctx.set_line_width(3);
        for i in 0..12 {
            let a = (i as f64 / 12.0) * PI * 2.0 - PI / 2.0;
            let r1 = w * 0.44;
            let r2 = if i % 3 == 0 { w * 0.36 } else { w * 0.40 };
            ctx.begin_path();
            ctx.move_to(cx + a.dcos() * r1, cy + a.dsin() * r1);
            ctx.line_to(cx + a.dcos() * r2, cy + a.dsin() * r2);
            ctx.stroke();
        }
        // Center dot.
        ctx.set_fill_style("#202020");
        ctx.begin_path();
        ctx.arc(cx, cy, 4, 0, PI * 2.0);
        ctx.fill();
    })
);
