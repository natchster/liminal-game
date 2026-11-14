//! `place_metal_lockers`
use crate::shared::t10_fill;
use game::assets::props::prop_part_as_modelled;
use game::prelude::*;
use placers_kit::geometry::put as t10_put;
use placers_kit::noise::fbm as t10_fbm;
use placers_kit::noise::mix_unclamped as t10_mix;
use placers_kit::noise::sstep as t10_ss;
use placers_kit::noise::vn as t10_vn;
use placers_kit::place::mount_slid as mount;
use placers_kit::place::vary as t10_vary;
use placers_kit::place::wall_candidates;
use placers_kit::place::wall_xz;

// Metal locker: 4% chance per level. A single-tier steel locker, 12 x
// 12 x 60 in on its 6 in legs (Blender batch `t10`), its welded frame, its
// door with six louvres top and bottom, its hinges and its recessed latch
// with a padlock staple, a riveted aluminium number plate; its paint
// scratched to the grey primer and the steel, rust coming through at its
// foot and its edges, grime where hands opened it.
pub(crate) fn place_metal_lockers(g: &mut Game, _dist: &[i32]) {
    g.pp.locker_positions = vec![]; // reset per level
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = choice(&candidates);
    let locker_colors = [0x606870, 0x506050, 0x403830, 0xd8b060];
    let col = choice(&locker_colors);
    let paint = LK_PAINT[locker_colors.iter().position(|&k| k == col).unwrap_or(0)];
    let locker = group();
    // origin = the door face; the body backs onto the wall
    let (wall_x, wall_z) = wall_xz(g, c, 0.300);
    mount(&locker, c, wall_x, wall_z, 0.0, 0.4);
    let p = locker.pos();
    let vary = t10_vary(p.x, p.z, 0.3);
    // the parts are modelled with their back at z = 0: on the wall
    let body = group();
    body.position().z = -0.300;
    locker.add(&body);
    let case = phong!({ map: &lk_paint_tex(vary, paint, false), specular: 0x161616, shininess: 18 });
    let door = phong!({ map: &lk_paint_tex(vary ^ 0x3d, paint, true), specular: 0x161616, shininess: 18 });
    t10_put(&body, "Real__lockerCase", &case);
    t10_put(&body, "Real__lockerDoor", &door);
    t10_put(&body, "Real__lockerLatch", &phong!({ color: 0x77797a, specular: 0x5a5a5a, shininess: 50 }));
    // the dark of its inside, seen through the louvres
    let dark = lambert!({ color: 0x0b0a09 });
    for y in [LK_DOOR_Y0 + LK_DOOR_H - 0.085 + 0.029, LK_DOOR_Y0 + 0.040 + 0.029] {
        let m = mesh(plane_geo!(0.17, 0.080), &dark);
        m.position().set(0, y, LK_DOOR_Z - 0.006);
        body.add(&m);
    }
    if let Some(plate) = t10_put(&body, "Real__lockerPlate", &lambert!({ map: &lk_plate_tex(100 + vary % 300) })) {
        plate.position().set(0, LK_DOOR_Y0 + LK_DOOR_H - 0.135, LK_DOOR_Z);
    }
    g.world.add(&locker);
    // Track for slam audio.
    let p = locker.pos();
    g.pp.locker_positions.push(v3(p.x, 0.8, p.z));
}

/// The locker (t10.py `LK_*`): its door's width, its bottom and height,
/// where its face is (z, from the back), where the latch is
const LK_DOOR_W: f64 = 0.305 - 2.0 * 0.022 - 2.0 * 0.0025;

const LK_DOOR_Y0: f64 = 0.152 + 0.048 + 0.0025;

const LK_DOOR_H: f64 = 1.524 - 0.032 - 0.048 - 2.0 * 0.0025;

const LK_DOOR_Z: f64 = 0.305 - 0.0015;

/// The paint (sRGB 0..1) for the old colours in their order: medium grey,
/// hunter green, dark brown, sand
const LK_PAINT: [[f64; 3]; 4] = [[0.42, 0.44, 0.45], [0.30, 0.37, 0.31], [0.26, 0.22, 0.18], [0.75, 0.69, 0.56]];

/// The locker's paint, as its front is mapped (u across, v up): the paint
/// mottled and faded, scratched to the primer and the steel, rust bleeding
/// from its foot and its edges, grime low down and round the latch
fn lk_paint_tex(vary: u32, paint: [f64; 3], door: bool) -> Tex {
    let (w, h) = (128.0, 640.0);
    canvas_tex_opt(w, h, false, move |ctx, w, h| {
        let mut r = SplitMix64::new(vary, 0x10c4);
        let n = 26 + (r.next() * 14.0) as usize;
        let scratches: Vec<((f64, f64), (f64, f64), f64)> = (0..n)
            .map(|_| {
                let (x, y) = (r.next(), r.next().powf(1.6));
                let a = r.range(-0.6, 0.6) + if r.next() < 0.5 { 0.0 } else { PI / 2.0 };
                let l = r.range(0.01, 0.12);
                ((x, y), (x + a.cos() * l, y + a.sin() * l * 0.2), r.range(0.3, 1.0))
            })
            .collect();
        let seed = (r.next() * 4.0e9) as u32;
        // the latch's place on the door's face (u, v)
        let latch = (0.5 + (LK_DOOR_W / 2.0 - 0.034) / LK_DOOR_W, 0.62);
        t10_fill(ctx, w, h, |u, v| {
            // in metres on the face, roughly
            let (x, y) = (u * 0.30, v * if door { 1.45 } else { 1.68 });
            let m = t10_fbm(seed, x * 8.0, y * 8.0, 4);
            let mut o = paint.map(|c| c * (0.9 + 0.16 * m));
            // faded and dulled
            o = t10_mix(o, [0.55, 0.53, 0.50], 0.12 + 0.08 * t10_fbm(seed ^ 3, x * 3.0, y * 2.0, 2));
            // the foot and the edges: rust come through the paint, bleeding down
            let edge = (u.min(1.0 - u) * 0.30).min(if door { v.min(1.0 - v) * 1.45 } else { 1.0 });
            let foot = if door { v * 1.45 } else { (v * 1.68 - 0.15).max(0.0) };
            let rn = t10_fbm(seed ^ 7, x * 30.0, y * 30.0, 4);
            let rust = (1.0 - t10_ss(0.0, 0.10, foot)) * t10_ss(0.42, 0.62, rn) + (1.0 - t10_ss(0.0, 0.012, edge)) * t10_ss(0.5, 0.65, rn);
            let spots = t10_ss(0.80, 0.86, t10_fbm(seed ^ 11, x * 22.0, y * 22.0, 3));
            let rust = (rust + spots * 0.8).min(1.0);
            o = t10_mix(o, [0.36, 0.20, 0.10], 0.85 * rust);
            // streaks below the rust spots
            let streak = t10_ss(0.72, 0.85, t10_fbm(seed ^ 11, x * 22.0, y * 22.0 + 0.6, 3))
                * t10_ss(0.5, 0.9, t10_vn(seed ^ 19, x * 90.0, y * 2.0));
            o = t10_mix(o, [0.40, 0.27, 0.16], 0.35 * streak);
            // grime: kicked at its foot, handled round its latch
            o = o.map(|c| c * (1.0 - 0.25 * (1.0 - t10_ss(0.0, 0.25, foot))));
            if door {
                let (du, dv) = ((u - latch.0) * 0.30, (v - latch.1) * 1.45);
                let hand = 1.0 - t10_ss(0.02, 0.12, (du * du + dv * dv * 0.5).sqrt());
                o = t10_mix(o, [0.16, 0.14, 0.12], 0.45 * hand * (0.6 + 0.4 * m));
            }
            o
        });
        // scratches: through to the grey primer, the deepest to the steel
        ctx.set_line_cap("round");
        for &(a, b, k) in &scratches {
            ctx.set_stroke_style(if k > 0.8 { "rgba(140,143,143,0.75)" } else { "rgba(158,156,148,0.6)" });
            ctx.set_line_width(0.6 + 0.8 * k);
            ctx.begin_path();
            ctx.move_to(a.0 * w, (1.0 - a.1) * h);
            ctx.line_to(b.0 * w, (1.0 - b.1) * h);
            ctx.stroke();
        }
    })
}

/// The number plate: stamped aluminium, its number filled black, dulled
fn lk_plate_tex(num: u32) -> Tex {
    canvas_tex_opt(96, 48, false, move |ctx, w, h| {
        ctx.set_fill_style("#7f807c");
        ctx.fill_rect(0, 0, w, h);
        let gr = ctx.create_linear_gradient(0, 0, w, h);
        gr.add_color_stop(0, "rgba(255,255,255,0.10)");
        gr.add_color_stop(1, "rgba(0,0,0,0.18)");
        ctx.set_fill_style(gr);
        ctx.fill_rect(0, 0, w, h);
        ctx.set_fill_style("rgba(24,22,20,0.9)");
        ctx.set_font("bold 30px Helvetica");
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        ctx.fill_text(&format!("{num}"), w / 2.0, h / 2.0 + 1.0);
        for _ in 0..60 {
            ctx.set_fill_style("rgba(60,56,50,0.12)");
            ctx.fill_rect(rnd() * w, rnd() * h, 2, 1);
        }
    })
}
