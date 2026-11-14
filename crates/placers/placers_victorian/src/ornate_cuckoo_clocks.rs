//! `place_ornate_cuckoo_clocks`
use game::assets::props::prop_part_as_modelled;
use game::prelude::*;
use placers_kit::geometry::put as t10_put;
use placers_kit::noise::fbm as t10_fbm;
use placers_kit::noise::mix_unclamped as t10_mix;
use placers_kit::noise::sstep as t10_ss;
use placers_kit::noise::vn as t10_vn;
use placers_kit::paint::fill as t10_fill;
use placers_kit::paint::opaque as t10_opaque;
use placers_kit::place::face_wall;
use placers_kit::place::vary as t10_vary;
use placers_kit::place::wall_tiles;

// Ornate cuckoo clock: 3% chance per level. A carved Black Forest
// cuckoo clock of the traditional kind (Blender batch `t10`): oak leaves
// and acorns carved round its dial, a bird with its wings spread on its
// top, the cuckoo's door shut; a dark wooden dial with bone numerals and
// hands, stopped; a carved leaf on its pendulum, its two cast pine-cone
// weights on brass chains. Linden stained dark, the stain rubbed from the
// carving's high points, dust in its hollows.
pub(crate) fn place_ornate_cuckoo_clocks(g: &mut Game, _dist: &[i32]) {
    g.pp.ornate_cuckoo_clock_positions.clear(); // reset per level
    if rnd() < 0.97 {
        return;
    }
    let wt = wall_tiles(g);
    if wt.is_empty() {
        return;
    }
    let (x, z, dx, dz) = wt[(rnd() * wt.len() as f64).floor() as usize];
    let cc = group();
    cc.rotation().y = face_wall(dx, dz);
    cc.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.015), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.015));
    let p = cc.pos();
    let vary = t10_vary(p.x, p.z, CK_DIAL_Y);
    // the parts are modelled with the wall at z = 0 and the dial's middle at
    // y = 0: on the wall, 15 mm behind the spot
    let body = group();
    body.position().set(0, CK_DIAL_Y, -0.015);
    cc.add(&body);
    t10_put(&body, "Real__cuckooCase", &lambert!({ map: &ck_wood_tex(vary, 0.0) }));
    t10_put(&body, "Real__cuckooCarving", &lambert!({ map: &ck_wood_tex(vary ^ 0x2b, 0.35) }));
    t10_put(&body, "Real__cuckooDial", &lambert!({ map: &ck_dial_tex(vary) }));
    // the hands, stopped where the old ones were (the old draws)
    let hour_ang = rnd() * PI * 2.0 - PI / 2.0;
    let min_ang = rnd() * PI * 2.0 - PI / 2.0;
    let bone = lambert!({ color: 0xc8b996 });
    for (name, ang, dz) in [("Real__cuckooHour", hour_ang, 0.0004), ("Real__cuckooMinute", min_ang, 0.0022)] {
        if let Some(h) = t10_put(&body, name, &bone) {
            h.position().set(0, 0, CK_DIAL_FACE + dz);
            h.rotation().z = ang - PI / 2.0;
        }
    }
    let brass = phong!({ color: 0x6e5a32, specular: 0x5a4a2a, shininess: 40 });
    t10_put(&body, "Real__cuckooChains", &brass);
    t10_put(&body, "Real__cuckooWeights", &phong!({ map: &ck_iron_tex(vary), specular: 0x1e1e1e, shininess: 20 }));
    // the pendulum hangs from its hook under the case
    let pendulum_group = group();
    pendulum_group.position().set(CK_PEND.0, CK_PEND.1, CK_PEND.2);
    t10_put(&pendulum_group, "Real__cuckooPendRod", &brass);
    t10_put(&pendulum_group, "Real__cuckooPendBob", &lambert!({ map: &ck_wood_tex(vary ^ 0x55, 0.35) }));
    body.add(&pendulum_group);
    // 50% chance: pendulum swing animator (tracked for cuckoo-call audio).
    let is_swinging = rnd() < 0.5;
    if is_swinging {
        let swing_phase = rnd() * PI * 2.0;
        let pg2 = pendulum_group.clone();
        g.animate(move |_g, _dt, t| {
            pg2.rotation().z = (t * PI * 1.4 + swing_phase).sin() * 0.20;
        });
    }
    g.world.add(&cc);
    if is_swinging {
        let p = cc.pos();
        g.pp.ornate_cuckoo_clock_positions.push(v3(p.x, 0, p.z));
    }
}

/// The clock (t10.py `CK_*`): where its dial's middle hangs (y), the
/// dial's face (z), the pendulum's hook under the case
const CK_DIAL_Y: f64 = 1.70;

const CK_DIAL_FACE: f64 = 0.008 + 0.110 + 0.004;

const CK_PEND: (f64, f64, f64) = (0.0, -0.020 - 0.100, 0.030);

/// Linden stained dark walnut, as the clock's front is mapped (u across, v
/// up): the stain's streaks, darker in the hollows and the dust lodged
/// there, rubbed lighter where it stands out (`rub`: the carving's more)
fn ck_wood_tex(vary: u32, rub: f64) -> Tex {
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        let seed = vary ^ 0xc0c;
        t10_fill(ctx, w, h, |u, v| {
            let g = t10_fbm(seed, u * 4.0, v * 30.0, 3);
            let fine = t10_vn(seed ^ 3, u * 90.0, v * 400.0);
            let mut o = [0.17, 0.095, 0.05].map(|c| c * (0.8 + 0.35 * g + 0.1 * fine));
            // the knife's facets and the stain rubbed off them
            let facet = t10_fbm(seed ^ 5, u * 40.0, v * 40.0, 2);
            o = t10_mix(o, [0.42, 0.29, 0.17], rub * t10_ss(0.55, 0.75, facet));
            // dust in the hollows, a grey bloom over all of it
            let dust = 0.04 + 0.12 * t10_ss(0.4, 0.75, t10_fbm(seed ^ 7, u * 14.0, v * 14.0, 3));
            t10_opaque(t10_mix(o, [0.40, 0.37, 0.33], dust))
        });
    })
}

/// The dial (100 mm, as its face is mapped): dark wood, a turned ring,
/// Roman numerals of bone set round it, the minute dots between
fn ck_dial_tex(vary: u32) -> Tex {
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        let seed = vary ^ 0xd1a1;
        t10_fill(ctx, w, h, |u, v| {
            let g = t10_fbm(seed, u * 3.0, v * 24.0, 3);
            let (x, y) = (u - 0.5, v - 0.5);
            let rr = (x * x + y * y).sqrt();
            let mut o = [0.17, 0.10, 0.06].map(|c| c * (0.8 + 0.4 * g));
            // the turned rings
            if (rr - 0.43).abs() < 0.006 || (rr - 0.25).abs() < 0.004 {
                o = o.map(|c| c * 0.6);
            }
            t10_opaque(t10_mix(o, [0.42, 0.40, 0.35], 0.12))
        });
        let (cx, cy) = (w / 2.0, h / 2.0);
        const NUM: [&str; 12] = ["XII", "I", "II", "III", "IIII", "V", "VI", "VII", "VIII", "IX", "X", "XI"];
        ctx.set_fill_style("rgba(206,192,156,0.95)");
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        ctx.set_font("bold 24px Times New Roman");
        for (k, n) in NUM.iter().enumerate() {
            let a = k as f64 * PI / 6.0;
            ctx.save();
            ctx.translate(cx + a.sin() * 86.0, cy - a.cos() * 86.0);
            ctx.rotate(a);
            ctx.scale(0.8, 1.0);
            ctx.fill_text(n, 0, 0);
            ctx.restore();
        }
        for k in 0..60 {
            if k % 5 == 0 {
                continue;
            }
            let a = k as f64 * PI / 30.0;
            ctx.begin_path();
            ctx.arc(cx + a.sin() * 112.0, cy - a.cos() * 112.0, 1.4, 0, 7);
            ctx.fill();
        }
        // the bone yellowed and grimed, unevenly
        let mut r = SplitMix64::new(vary, 0xd1a2);
        for _ in 0..6 {
            let (x, y, rr) = (r.next() * w, r.next() * h, 20.0 + r.next() * 50.0);
            let gr = ctx.create_radial_gradient(x, y, 0, x, y, rr);
            gr.add_color_stop(0, "rgba(60,44,24,0.25)");
            gr.add_color_stop(1, "rgba(60,44,24,0)");
            ctx.set_fill_style(gr);
            ctx.fill_rect(x - rr, y - rr, rr * 2.0, rr * 2.0);
        }
    })
}

/// Cast iron gone dull, rust coming through (the weights)
fn ck_iron_tex(vary: u32) -> Tex {
    canvas_tex_opt(64, 128, false, move |ctx, w, h| {
        let seed = vary ^ 0x1e0;
        t10_fill(ctx, w, h, |u, v| {
            let n = t10_fbm(seed, u * 8.0, v * 16.0, 3);
            let rust = t10_ss(0.55, 0.75, t10_fbm(seed ^ 3, u * 12.0, v * 24.0, 3));
            t10_opaque(t10_mix([0.08 + 0.04 * n, 0.075 + 0.035 * n, 0.07 + 0.03 * n], [0.30, 0.17, 0.09], rust * 0.8))
        });
    })
}
