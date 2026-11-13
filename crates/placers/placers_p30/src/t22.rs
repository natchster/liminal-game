//! Batch `t22`'s wet floor sign (`docs/design/PROPS.md` P3): the core props' sign
//! (batch `r01`, `core_props::build_wet_floor_sign`), placed by this
//! placer as the core places it: the common 2-sided A-frame, its two
//! moulded panels hinged at the top and splayed 0.30 m at the foot, each
//! printed CAUTION / WET FLOOR over the slipping figure. It keeps the old
//! placer's draws, call for call.
//!
//! The sign stands on one floor in five, and it stays instanced as the old
//! one was: its print is painted into the panel's own texture (the core
//! lays it on as a transparent decal), shared by every sign, and its hinge
//! is split between the panels (`Real__wetSignHingeHalf`, batch `t22`), so
//! every part of it is a mesh that repeats.
use crate::{open_tiles, pick};
use game::prelude::*;
use std::cell::OnceCell;

/// The panel as modelled (`r01.py`): its length, its bounds across (x) and
/// where its printed face is (z)
const PANEL_H: f64 = 0.655;
const PANEL_X: (f64, f64) = (-0.1447, 0.1447);
const FACE_Z: f64 = 0.0195;
/// The print as the core lays it on the panel: its size and its middle's
/// height (`build_wet_floor_sign`)
const PRINT: (f64, f64, f64) = (0.222, 0.47, 0.2925);

// Wet floor sign: 20% chance per level, 1 yellow A-frame sign: the
// core's sign (r01), its wear one of two (the old turn's draw), standing
// where it was left.
pub(crate) fn place_wet_floor_signs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.80 {
        return;
    } // 20% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let pk = pick(&open);
    let px = g.wx(pk.0) + (rnd() - 0.5) * 1.8;
    let pz = g.wz(pk.1) + (rnd() - 0.5) * 1.8;
    // the old turn's draw: its turn, and in its fine digits which of the
    // two signs (how worn)
    let turn = rnd();
    let worn = ((turn * 4096.0) as u32) & 1;
    let Some(kit) = sign_kit() else { return };
    let sign = group();
    let splay = (0.15f64 / PANEL_H).asin();
    let hinge_y = PANEL_H * splay.cos();
    for side in [0.0, PI] {
        // each panel hangs from the hinge, its foot out to its own side,
        // and brings its half of the hinge
        let leaf = group();
        leaf.rotation().y = side;
        leaf.position().y = hinge_y;
        let swing = group();
        swing.rotation().x = -splay;
        let panel = mesh(kit.panel.clone(), &kit.plastic[worn as usize]);
        panel.position().y = -PANEL_H;
        swing.add(&panel);
        leaf.add(&swing);
        leaf.add(&mesh(kit.hinge.clone(), &kit.hinge_mat));
        sign.add(&leaf);
    }
    sign.position().set(px, 0, pz);
    sign.rotation().y = turn * PI * 2.0;
    g.world.add(&sign);
}

/// What every sign shares: the panel (its printed face laid with the whole
/// panel's texture), the hinge's half, the plastic of each sign and the
/// hinge's
struct SignKit {
    panel: Geo,
    hinge: Geo,
    plastic: [Mat; 2],
    hinge_mat: Mat,
}

fn sign_kit() -> Option<std::rc::Rc<SignKit>> {
    thread_local! { static K: OnceCell<Option<std::rc::Rc<SignKit>>> = const { OnceCell::new() }; }
    K.with(|c| {
        c.get_or_init(|| {
            let props = game::assets::props::prop_part_as_modelled;
            Some(std::rc::Rc::new(SignKit {
                panel: panel_geo(&props("Real__wetSignPanel")?),
                hinge: props("Real__wetSignHingeHalf")?,
                plastic: [lambert!({ map: &panel_tex(0) }), lambert!({ map: &panel_tex(1) })],
                hinge_mat: lambert!({ color: 0x9a7c1c }),
            }))
        })
        .clone()
    })
}

/// A copy of the panel whose printed face takes the texture over the whole
/// panel (u across it, v up it, as it stands), and whose every other face
/// (the rim, the ribs, the back, the slot) takes the plain plastic at the
/// texture's edge
fn panel_geo(src: &Geo) -> Geo {
    let geo = src.duplicate();
    {
        let mut gg = geo.g();
        let n = gg.position.len() / 3;
        let mut uv = vec![0f32; n * 2];
        for i in 0..n {
            let (x, y, z) = (f64::from(gg.position[i * 3]), f64::from(gg.position[i * 3 + 1]), f64::from(gg.position[i * 3 + 2]));
            let nz = f64::from(gg.normal[i * 3 + 2]);
            let v = y / PANEL_H;
            let u = if nz > 0.6 && z > FACE_Z - 0.001 { (x - PANEL_X.0) / (PANEL_X.1 - PANEL_X.0) } else { 0.012 };
            uv[i * 2] = u as f32;
            uv[i * 2 + 1] = v as f32;
        }
        gg.uv = uv;
        gg.touch();
    }
    geo
}

/// The panel's face: the moulded yellow plastic (its flow marks, the grime
/// of the mop water at its foot, scuffs from shoes and the bucket), and the
/// core's print on it (`wet_sign_print_tex`'s layout: CAUTION over the
/// slipping figure, WET FLOOR and the same in two more languages, black),
/// rubbed thin at its edges and scratched; `worn` 1 the more worn of two.
/// Its strokes come from a stream of its own seeded by `worn` (never the
/// level's), so a sign is the same whichever floor first made it
fn panel_tex(worn: u32) -> Tex {
    canvas_tex_opt(320, 720, false, move |ctx, w, h| {
        let mut r = Lcg(0x5167_0000 + worn);
        let wear = 1.0 + 0.6 * f64::from(worn);
        // the plastic
        fill(ctx, w, h, |u, v| {
            let y = v * PANEL_H;
            let flow = fbm(11 + worn, u * 6.0, v * 14.0, 3);
            let c = mix([0.80, 0.645, 0.165], [0.72, 0.57, 0.14], flow * 1.4 - 0.3);
            // the mop water's grime, risen up its foot
            let tide = 0.06 + 0.03 * vn(23 + worn, u * 9.0, 0.0);
            let grime = (1.0 - sstep(tide * 0.4, tide, y)) * 0.55 + 0.12 * (1.0 - sstep(0.0, 0.30, y));
            let c = mix(c, [0.36, 0.30, 0.16], grime * wear.min(1.4));
            [c[0], c[1], c[2], 1.0]
        });
        // the print, on a layer of its own, then rubbed and scratched
        let mut pr = Canvas::new(256, 544);
        print(&mut pr, 256.0, 544.0);
        pr.set_global_composite_operation("destination-out");
        for _ in 0..(70.0 * wear) as i32 {
            let (x, y) = (r.f() * 256.0, r.f() * 544.0);
            pr.set_global_alpha(0.12 + 0.35 * r.f());
            pr.begin_path();
            pr.ellipse(x, y, 2.0 + r.f() * 9.0, 1.0 + r.f() * 4.0, r.f() * 3.0, 0, 7);
            pr.fill();
        }
        pr.set_line_width(1.2);
        for _ in 0..(36.0 * wear) as i32 {
            let (x, y) = (r.f() * 256.0, r.f() * 544.0);
            let (a, l) = (r.f() * 6.3, 8.0 + r.f() * 40.0);
            pr.set_global_alpha(0.4 + 0.5 * r.f());
            pr.begin_path();
            pr.move_to(x, y);
            pr.line_to(x + a.cos() * l, y + a.sin() * l);
            pr.stroke();
        }
        // its foot, where the mop and the shoes rub it
        let grad = pr.create_linear_gradient(0, 544.0 - 70.0, 0, 544);
        grad.add_color_stop(0, "rgba(0,0,0,0)");
        grad.add_color_stop(1, "rgba(0,0,0,0.6)");
        pr.set_global_alpha(1);
        pr.set_fill_style(grad);
        pr.fill_rect(0, 544.0 - 70.0, 256, 70);
        pr.set_global_composite_operation("source-over");
        // laid where the core lays its decal
        let pw = PANEL_X.1 - PANEL_X.0;
        let x0 = (PRINT.0 / -2.0 - PANEL_X.0) / pw * w;
        let top = (1.0 - (PRINT.2 + PRINT.1 / 2.0) / PANEL_H) * h;
        ctx.draw_image_scaled(&pr, x0, top, PRINT.0 / pw * w, PRINT.1 / PANEL_H * h);
        // scuffs over both: shoes' rubber, the bucket's knocks
        for _ in 0..(14.0 * wear) as i32 {
            let (x, y) = (r.f() * w, h * (0.45 + 0.55 * r.f()));
            let (a, l) = (r.f() * 0.8 - 0.4, 6.0 + r.f() * 26.0);
            ctx.set_stroke_style(if r.f() < 0.7 { "rgba(30,26,20,0.45)" } else { "rgba(235,215,150,0.35)" });
            ctx.set_line_width(1.0 + r.f() * 2.5);
            ctx.begin_path();
            ctx.move_to(x, y);
            ctx.line_to(x + a.cos() * l, y + a.sin() * l);
            ctx.stroke();
        }
    })
}

/// The core sign's print (`wet_sign_print_tex`), black on transparent at
/// 256 x 544 px for the 0.222 x 0.47 m it covers
fn print(ctx: &mut Canvas, w: f64, _h: f64) {
    ctx.set_fill_style("#141210");
    ctx.set_text_align("center");
    ctx.set_text_baseline("alphabetic");
    ctx.set_font("bold 54px Helvetica Neue");
    ctx.fill_text_max("CAUTION", w / 2.0, 66, w - 16.0);
    // the slipping figure (ISO 7010 W011's person, in solid strokes)
    ctx.set_stroke_style("#141210");
    ctx.set_line_cap("round");
    ctx.set_line_join("round");
    ctx.set_line_width(19);
    let (cx0, cy0) = (w / 2.0 + 6.0, 255.0);
    ctx.begin_path();
    ctx.arc(cx0 + 34.0, cy0 - 104.0, 19, 0, 7);
    ctx.fill();
    for line in [
        [(cx0 + 22.0, cy0 - 74.0), (cx0 - 8.0, cy0 + 4.0), (cx0 - 8.0, cy0 + 4.0)],
        [(cx0 - 8.0, cy0 + 4.0), (cx0 + 40.0, cy0 + 22.0), (cx0 + 72.0, cy0 - 4.0)],
        [(cx0 - 8.0, cy0 + 4.0), (cx0 - 26.0, cy0 + 50.0), (cx0 - 6.0, cy0 + 72.0)],
        [(cx0 - 64.0, cy0 - 70.0), (cx0 + 12.0, cy0 - 50.0), (cx0 + 78.0, cy0 - 92.0)],
    ] {
        ctx.begin_path();
        ctx.move_to(line[0].0, line[0].1);
        ctx.line_to(line[1].0, line[1].1);
        ctx.line_to(line[2].0, line[2].1);
        ctx.stroke();
    }
    // the floor, and the wet on it
    ctx.set_line_width(7);
    ctx.begin_path();
    ctx.move_to(24, cy0 + 86.0);
    ctx.line_to(w - 24.0, cy0 + 86.0);
    ctx.stroke();
    ctx.set_line_width(4);
    for k in 0..3 {
        let x = 52.0 + f64::from(k) * 58.0;
        ctx.begin_path();
        ctx.move_to(x, cy0 + 100.0);
        ctx.bezier_curve_to(x + 10.0, cy0 + 94.0, x + 20.0, cy0 + 106.0, x + 30.0, cy0 + 100.0);
        ctx.stroke();
    }
    ctx.set_font("bold 40px Helvetica Neue");
    ctx.fill_text_max("WET FLOOR", w / 2.0, 418, w - 20.0);
    ctx.set_font("bold 25px Helvetica Neue");
    ctx.fill_text_max("PISO MOJADO", w / 2.0, 458, w - 20.0);
    ctx.set_font("bold 19px Helvetica Neue");
    ctx.fill_text_max("ATTENTION SOL GLISSANT", w / 2.0, 490, w - 20.0);
}

/// A small stream of the painter's own (never the level's nor the native)
struct Lcg(u32);
impl Lcg {
    fn f(&mut self) -> f64 {
        self.0 = self.0.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let mut h = self.0;
        h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
        h ^= h >> 13;
        f64::from(h) / 4_294_967_296.0
    }
}

use placers_kit::noise::fbm;
use placers_kit::noise::hash;
use placers_kit::noise::mix;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
/// Fills the canvas pixel by pixel on every core: `f(u, v)` (u right, v up,
/// both 0..1, as the texture is laid) gives straight sRGB and alpha
fn fill(ctx: &mut Canvas, w: f64, h: f64, f: impl Fn(f64, f64) -> [f64; 4] + Sync) {
    let iw = w as usize;
    let mut img = ctx.create_image_data(w, h);
    par_rows(&mut img.data, iw * 4, |j, out| {
        for i in 0..iw {
            let c = f((i as f64 + 0.5) / w, 1.0 - (j as f64 + 0.5) / h);
            for k in 0..4 {
                out[i * 4 + k] = (c[k].clamp(0.0, 1.0) * 255.0).round() as u8;
            }
        }
    });
    ctx.put_image_data(&img, 0, 0);
}
