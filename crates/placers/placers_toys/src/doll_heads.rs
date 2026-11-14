//! `place_doll_heads`
use crate::shared::vinyl_tex;
use game::prelude::*;
use placers_kit::geometry::put;
use placers_kit::noise::fbm;
use placers_kit::noise::hash;
use placers_kit::noise::mix;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::fill;
use placers_kit::place::open_tiles;
use placers_kit::place::pick_hex;
use placers_kit::place::salt;

// A doll's head: 3% chance per level. A vinyl baby doll's head
// (batch r03's: 10 cm across, moulded hair, sleep eyes with lashes, the
// mouth open for its bottle) come off at its neck: the vinyl rolled in at
// the rim, the hollow dark inside. Lying on the back of its head as the
// doll lay, its lids down (the left stuck half open), or standing on its
// neck, its eyes open (the left stuck half shut); yellowed, grimed, the
// paint rubbed.
pub(crate) fn place_doll_heads(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    // the old hair colours: the paint of its moulded hair
    let hair_colors = [0x604020, 0x201810, 0xd8a040, 0x8a2020];
    let hair = pick_hex(&hair_colors);
    let lying = rnd() < 0.60;
    let px = g.wx(x) + (rnd() - 0.5) * 2.4;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.4;
    let turn = rnd() * PI * 2.0;
    let k = salt(px, pz, 494);
    let vinyl = lambert!({ map: &vinyl_tex(k, 0xd6b19a) });
    let doll = group();
    // the parts r03 laid into its lying body's frame
    let skull = group();
    put(&skull, "Real__dollHead", &vinyl);
    put(
        &skull,
        "Real__dollHair",
        &lambert!({ map: &hair_tex(k, hair), polygonOffset: true, polygonOffsetFactor: -1, polygonOffsetUnits: -1 }),
    );
    // and those in the head's own frame (face +z, crown +y)
    let head = group();
    put(&head, "Real__dollFace", &lambert!({ map: &face_tex(k) }));
    let eye = phong!({ map: &eye_tex(k), specular: 0x2a2a2a, shininess: 70 });
    put(&head, "Real__dollEyeR", &eye);
    put(&head, "Real__dollEyeL", &eye);
    let lid = lambert!({ color: 0xbf9a86 });
    let lash = lambert!({ color: 0x3a312b, side: DoubleSide });
    if lying {
        put(&head, "Real__dollLids", &lid);
        put(&head, "Real__dollLashes", &lash);
    } else {
        put(&head, "Real__dollHeadLidsUp", &lid);
        put(&head, "Real__dollHeadLashesUp", &lash);
    }
    put(&head, "Real__dollHeadNeckLip", &vinyl);
    put(&head, "Real__dollHeadNeckHole", &lambert!({ color: 0x1c1716 }));
    if lying {
        // as the doll lies: on the back of its head, rolled toward its right
        head.position().set(DOLL_NECK.0, DOLL_NECK.1, DOLL_NECK.2);
        head.rotation().set(DOLL_TURN - PI / 2.0, 0, -PI / 2.0);
        doll.add(&skull);
        doll.add(&head);
    } else {
        // standing on its neck: the lying frame's parts brought back into
        // the head's (the inverse of r03's laying down)
        let a = group();
        a.rotation().z = PI / 2.0;
        let b = group();
        b.rotation().x = PI / 2.0 - DOLL_TURN;
        skull.position().set(-DOLL_NECK.0, -DOLL_NECK.1, -DOLL_NECK.2);
        b.add(&skull);
        a.add(&b);
        head.add(&a);
        head.position().y = NECK_LIFT;
        doll.add(&head);
    }
    doll.position().set(px, 0, pz);
    doll.rotation().y = turn;
    g.world.add(&doll);
}

/// r03's head frame in its lying body's (`game/rubber_duck_baby_doll_and_hole.py` DOLL_NECK, DOLL_TURN):
/// where the head's own parts are laid when it lies as the doll's does
const DOLL_NECK: (f64, f64, f64) = (0.050, 0.0592, 0.0);

const DOLL_TURN: f64 = -0.30;

/// How far a head standing on its neck stands up: its neck's rim under the
/// head frame's origin (`toys/soft_toys_and_dolls.py` `head_neck_lip`)
const NECK_LIFT: f64 = 0.0139;

/// Moulded hair's paint (`col`): combed in fine streaks, rubbed back to the
/// vinyl on the crown and the waves' crests, dust in the grooves
fn hair_tex(salt: u32, col: u32) -> Tex {
    let p = [f64::from((col >> 16) & 255) / 255.0, f64::from((col >> 8) & 255) / 255.0, f64::from(col & 255) / 255.0];
    // the paint of a doll's hair, not a toy's: muted toward a brown
    let p = mix(p, [0.42, 0.33, 0.24], 0.35);
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        let seed = salt ^ 0x4a1;
        fill(ctx, w, h, |u, v| {
            let streak = fbm(seed, u * 60.0, v * 6.0, 3);
            let mut c = mix(p, [p[0] * 0.6, p[1] * 0.6, p[2] * 0.6], sstep(0.45, 0.75, streak) * 0.6);
            c = mix(
                c,
                [p[0] * 1.2 + 0.05, p[1] * 1.2 + 0.04, p[2] * 1.15 + 0.03],
                sstep(0.6, 0.8, fbm(seed ^ 2, u * 30.0, v * 4.0, 2)) * 0.3,
            );
            // rubbed through to the vinyl
            let rub = fbm(seed ^ 3, u * 16.0, v * 16.0, 3);
            c = mix(c, [0.72, 0.58, 0.48], sstep(0.70, 0.78, rub) * 0.55);
            // dust in the grooves
            c = mix(c, [0.48, 0.45, 0.40], sstep(0.55, 0.75, fbm(seed ^ 9, u * 4.0, v * 4.0, 2)) * 0.25);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The face part's bounds in the head's frame (x across, y up; game/rubber_duck_baby_doll_and_hole.py,
/// `Real__dollFace`): its paint is laid straight on along z
const FACE_LO: (f64, f64) = (-0.0501, -0.0125);

const FACE_HI: (f64, f64) = (0.0501, 0.0961);

fn face_px(x: f64, y: f64, w: f64, h: f64) -> (f64, f64) {
    let u = (x - FACE_LO.0) / (FACE_HI.0 - FACE_LO.0);
    let v = (y - FACE_LO.1) / (FACE_HI.1 - FACE_LO.1);
    (u * w, (1.0 - v) * h)
}

/// The doll's face, as r03's (the same mould, the same factory's paint),
/// worn its own way: blush sprayed and rubbed off the cheeks' crowns, fine
/// brows, painted lower lashes, the lips round the bottle's hole, the
/// nostrils; dirt in the creases and specks, a grey scuff, paint rubbed
/// off the nose's tip
fn face_tex(salt: u32) -> Tex {
    canvas_tex_opt(256, 288, false, move |ctx, w, h| {
        let seed = salt ^ 0xfa;
        let k = w / (FACE_HI.0 - FACE_LO.0); // px per metre
        let px = |x: f64, y: f64| face_px(x, y, w, h);
        fill(ctx, w, h, |u, v| {
            let b = [0.84, 0.69, 0.60];
            let n = fbm(seed, u * 5.0, v * 5.0, 4);
            let mut c = mix(b, [b[0] * 0.84, b[1] * 0.78, b[2] * 0.70], sstep(0.45, 0.8, n) * 0.45);
            let grime = fbm(seed ^ 5, u * 3.0, v * 3.0, 3);
            c = mix(c, [0.40, 0.33, 0.27], sstep(0.6, 0.85, grime) * 0.25);
            let speck = vn(seed ^ 7, u * 80.0, v * 90.0);
            if speck > 0.88 {
                c = mix(c, [0.36, 0.30, 0.24], (speck - 0.88) * 4.0);
            }
            [c[0], c[1], c[2], 1.0]
        });
        let blot = |ctx: &mut Canvas, x: f64, y: f64, r: f64, c0: &str, c1: &str| {
            let gr = ctx.create_radial_gradient(x, y, 0, x, y, r);
            gr.add_color_stop(0, c0);
            gr.add_color_stop(1, c1);
            ctx.set_fill_style(gr);
            ctx.fill_rect(x - r, y - r, r * 2.0, r * 2.0);
        };
        for s in [-1.0, 1.0] {
            let (cx0, cy0) = px(s * 0.025, 0.038);
            blot(ctx, cx0, cy0, 0.016 * k, "rgba(190,116,100,0.36)", "rgba(190,116,100,0)");
            blot(ctx, cx0 + s * 2.0, cy0 - 3.0, 0.006 * k, "rgba(214,180,160,0.4)", "rgba(214,180,160,0)");
            ctx.set_stroke_style("rgba(118,86,60,0.5)");
            ctx.set_line_width(1.2);
            for i in 0..9 {
                let t = f64::from(i) / 8.0;
                let x = s * (0.008 + 0.022 * t);
                let y = 0.0715 + 0.0035 * (1.0 - (2.0 * t - 0.8).powi(2));
                let (a, b) = px(x, y);
                ctx.begin_path();
                ctx.move_to(a, b + 1.0);
                ctx.line_to(a + s * 3.0, b - 1.4);
                ctx.stroke();
            }
            ctx.set_stroke_style("rgba(78,56,42,0.55)");
            ctx.set_line_width(0.9);
            for i in 0..6 {
                let t = f64::from(i) / 5.0;
                let x = s * (0.0125 + 0.0135 * t);
                let (a, b) = px(x, 0.0532 - 0.0012 * (1.0 - (2.0 * t - 1.0).powi(2)) + s * (x - s * 0.019) * 0.1);
                ctx.begin_path();
                ctx.move_to(a, b);
                ctx.line_to(a + s * 1.2, b + 3.2);
                ctx.stroke();
            }
            let (ex, ey) = px(s * 0.019, 0.0585);
            ctx.set_stroke_style("rgba(80,62,48,0.4)");
            ctx.set_line_width(2.4);
            ctx.begin_path();
            ctx.ellipse(ex, ey, 0.0102 * k, 0.0060 * k, 0, 0, 7);
            ctx.stroke();
            let (a, b) = px(s * 0.0034, 0.0442);
            ctx.set_fill_style("rgba(96,58,48,0.6)");
            ctx.begin_path();
            ctx.ellipse(a, b, 1.6, 1.1, s * 0.4, 0, 7);
            ctx.fill();
        }
        // the lips, worn pale at their crowns, round the dark bottle hole
        let (mx, my) = px(0.0, 0.0333);
        ctx.set_fill_style("rgba(160,98,90,0.5)");
        ctx.begin_path();
        ctx.ellipse(mx, my, 0.0060 * k, 0.0051 * k, 0, 0, 7);
        ctx.fill();
        blot(ctx, mx, my - 0.0042 * k, 0.0022 * k, "rgba(214,170,150,0.5)", "rgba(214,170,150,0)");
        ctx.set_fill_style("rgba(48,24,22,0.95)");
        ctx.begin_path();
        ctx.ellipse(mx, my, 0.0027 * k, 0.0027 * k, 0, 0, 7);
        ctx.fill();
        // the nose's tip rubbed, a grey scuff where it lay on its face once
        let (nx, ny) = px(0.0, 0.047);
        blot(ctx, nx, ny, 0.004 * k, "rgba(232,212,196,0.45)", "rgba(232,212,196,0)");
        let sx = if hash(salt, 9, 9) < 0.5 { -1.0 } else { 1.0 };
        let (gx, gy) = px(sx * (0.020 + 0.012 * hash(salt, 1, 9)), 0.020 + 0.03 * hash(salt, 2, 9));
        blot(ctx, gx, gy, 0.012 * k, "rgba(70,64,58,0.34)", "rgba(70,64,58,0)");
    })
}

/// A sleep eye: an ivory ball yellowed, its iris (grey-blue or brown, by
/// the head) with a darker ring and streaks, the pupil black (the iris
/// faces +z: a quarter of the way along)
fn eye_tex(salt: u32) -> Tex {
    let brown = hash(salt, 7, 1) < 0.35;
    canvas_tex_opt(256, 128, false, move |ctx, w, h| {
        ctx.set_fill_style("#cfc4ad");
        ctx.fill_rect(0, 0, w, h);
        let (cx0, cy0) = (w * 0.25, h * 0.5);
        let gr = ctx.create_radial_gradient(cx0, cy0, 0, cx0, cy0, 21);
        if brown {
            gr.add_color_stop(0, "#3e2c1e");
            gr.add_color_stop(0.55, "#6a4c32");
            gr.add_color_stop(0.85, "#4e3826");
            gr.add_color_stop(1, "#241a12");
        } else {
            gr.add_color_stop(0, "#3d5468");
            gr.add_color_stop(0.55, "#62809a");
            gr.add_color_stop(0.85, "#4b677e");
            gr.add_color_stop(1, "#26323e");
        }
        ctx.set_fill_style(gr);
        ctx.begin_path();
        ctx.ellipse(cx0, cy0, 21, 21, 0, 0, 7);
        ctx.fill();
        ctx.set_stroke_style("rgba(28,30,34,0.45)");
        ctx.set_line_width(1);
        for i in 0..28 {
            let a = f64::from(i) / 28.0 * 6.283;
            ctx.begin_path();
            ctx.move_to(cx0 + a.cos() * 9.0, cy0 + a.sin() * 9.0);
            ctx.line_to(cx0 + a.cos() * 19.0, cy0 + a.sin() * 19.0);
            ctx.stroke();
        }
        ctx.set_fill_style("#1c1a1a");
        ctx.begin_path();
        ctx.ellipse(cx0, cy0, 8.5, 8.5, 0, 0, 7);
        ctx.fill();
    })
}
