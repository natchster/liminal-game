//! The doors in walls, made by the asset pipeline (`docs/plan/DOORS.md`;
//! their dossiers, `assets/src/props/<door>/`, hold every measurement's
//! source; their parts, `assets/src/blender/props/game/doors.py`): real
//! doors at their real sizes, in what they are made of, their surfaces
//! painted from their palettes on streams of their own (`canvas_tex`), and
//! no glow on anything that is not a light. Each is made in its own frame:
//! x across its face, y up from the floor, z out of the wall toward you,
//! its origin 6 cm in front of the wall face (`World::wall_adj_spot`).
use crate::assets::props::prop_part_as_modelled;
use crate::base::records::WallSpot;
use crate::placers::kit::canvas_tex_opt;
use crate::prelude::*;
use crate::visuals::textures::{TextTexOpts, text_tex};

/// The wall's face, in a door's frame
pub const WALL_Z: f64 = -0.06;
/// The exit door's opening (3'0" x 7'0") and its leaf's foot (the undercut)
pub const EXIT_OPEN_W: f64 = 0.914;
pub const EXIT_OPEN_H: f64 = 2.134;
pub const UNDERCUT: f64 = 0.019;
/// The exit sign's middle, and its lit face (in the housing's recess)
pub const SIGN_Y: f64 = 2.47;
const SIGN_FACE: (f64, f64) = (0.40, 0.20);
const SIGN_FACE_Z: f64 = WALL_Z + 0.0495;

/// A part as modelled, or the box filling its frame when the parts are off
fn part(name: &str) -> Geo {
    prop_part_as_modelled(name).unwrap_or_else(|| panic!("no prop part {name} (cargo xtask assets props)"))
}

fn add(g: &Obj, geo: &Geo, mat: &Mat) -> Obj {
    let m = mesh(geo, mat);
    g.add(&m);
    m
}

thread_local! {
    static VENEER: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() };
    static FRAME: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() };
    static ENAMEL: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() };
    static ENAMEL_PULL: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() };
    static SIGN: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() };
}

/// Red oak, plain sliced, under a clear lacquer gone amber
/// (`assets/src/props/exit_door/references.toml`, its palette), as the
/// leaf's face takes it whole (u across it from the hinge side, the
/// texture's top its head): three leaves of veneer slip-matched, each a
/// cathedral figure up its middle and straight grain to its edges, the
/// open pores along the grain; worn grey and dark where hands pushed it
/// (the latch edge, the lever's height, above it where a palm pushes), and
/// scuffed low over the kick plate where feet did
fn veneer_tex() -> Tex {
    VENEER.with(|c| {
        c.get_or_init(|| {
            canvas_tex_opt(512, 1024, false, |ctx, w, h| {
                let g = ctx.create_linear_gradient(0, 0, w, 0);
                g.add_color_stop(0.0, "#9a743f");
                g.add_color_stop(0.5, "#a27b44");
                g.add_color_stop(1.0, "#966f3d");
                ctx.set_fill_style(&g);
                ctx.fill_rect(0, 0, w, h);
                let leaves = 3;
                let lw = w / leaves as f64;
                for i in 0..leaves {
                    let x0 = i as f64 * lw;
                    let cx = x0 + lw * (0.42 + rnd() * 0.16);
                    // the leaf's own shade
                    ctx.set_fill_style(if i % 2 == 0 { "rgba(70,40,15,0.05)" } else { "rgba(255,210,150,0.035)" });
                    ctx.fill_rect(x0, 0, lw, h);
                    // straight grain toward its edges
                    for _ in 0..80 {
                        let mut x = x0 + rnd() * lw;
                        let dark = rnd();
                        ctx.set_stroke_style(format!("rgba(64,38,18,{:.3})", 0.04 + dark * 0.11));
                        ctx.set_line_width(0.6 + rnd() * 1.4);
                        ctx.begin_path();
                        ctx.move_to(x, 0);
                        let mut y = 0.0;
                        while y < h {
                            y += 48.0;
                            x += rnd() * 1.6 - 0.8;
                            ctx.line_to(x, y);
                        }
                        ctx.stroke();
                    }
                    // the cathedral figure: latewood bands arched up the
                    // middle, round-topped, flaring down into straight
                    // grain (a parabola: x = hw * sqrt(t)), wavering, unevenly
                    // spaced, drifting off the leaf's middle as they go up
                    let n = 6 + (rnd() * 4.0) as usize;
                    let top = h * (0.05 + rnd() * 0.14);
                    let drift = (rnd() - 0.5) * 6.0;
                    let mut f = 0.0;
                    for k in 0..n {
                        f += (0.6 + rnd() * 0.8) / n as f64;
                        let apex = top + (h * 0.7 - top) * f.min(1.0);
                        let hw = lw * (0.16 + 0.22 * rnd()) * (0.6 + f);
                        let ax = cx + drift * k as f64;
                        let ph = rnd() * 6.0;
                        for (col, lw2) in [("rgba(66,38,16,0.035)", 9.0), ("rgba(66,38,16,0.05)", 4.5), ("rgba(58,32,12,0.075)", 1.8)] {
                            ctx.set_stroke_style(col);
                            ctx.set_line_width(lw2);
                            for side in [-1.0, 1.0] {
                                ctx.begin_path();
                                let steps = 36;
                                for j in 0..=steps {
                                    let t = j as f64 / steps as f64;
                                    let y = apex + t * (h - apex + 10.0);
                                    let x = ax + side * hw * t.sqrt() + (y * 0.013 + ph).dsin() * 2.2;
                                    if j == 0 {
                                        ctx.move_to(ax, apex);
                                    } else {
                                        ctx.line_to(x, y);
                                    }
                                }
                                ctx.stroke();
                            }
                        }
                    }
                    // the joint between two leaves
                    if i > 0 {
                        ctx.set_fill_style("rgba(50,28,12,0.18)");
                        ctx.fill_rect(x0 - 0.5, 0, 1.0, h);
                    }
                }
                // the open pores, short dashes along the grain
                ctx.set_fill_style("rgba(52,30,14,0.16)");
                for _ in 0..3200 {
                    ctx.fill_rect(rnd() * w, rnd() * h, 0.8, 2.0 + rnd() * 5.0);
                }
                // the lacquer: amber, deeper toward the foot
                let l = ctx.create_linear_gradient(0, 0, 0, h);
                l.add_color_stop(0.0, "rgba(255,200,120,0.05)");
                l.add_color_stop(0.8, "rgba(120,70,20,0.06)");
                l.add_color_stop(1.0, "rgba(60,35,15,0.16)");
                ctx.set_fill_style(&l);
                ctx.fill_rect(0, 0, w, h);
                // hands: round the lever (0.92 across, 1024 mm up of 2111
                // from 19 mm: 0.52 down the texture), and where a palm pushes
                // above it
                for (x, y, r, col) in [
                    (0.92, 0.52, 0.16, "rgba(58,44,30,0.34)"),
                    (0.86, 0.38, 0.13, "rgba(70,55,40,0.24)"),
                    (0.90, 0.45, 0.10, "rgba(150,135,115,0.10)"),
                ] {
                    let (x, y, r) = (x * w, y * h, r * w);
                    let gr = ctx.create_radial_gradient(x, y, 0, x, y, r);
                    gr.add_color_stop(0.0, col);
                    gr.add_color_stop(1.0, "rgba(0,0,0,0)");
                    ctx.set_fill_style(&gr);
                    ctx.fill_rect(x - r, y - r, r * 2.0, r * 2.0);
                }
                // the latch edge, handled all its height
                let e = ctx.create_linear_gradient(w * 0.95, 0, w, 0);
                e.add_color_stop(0.0, "rgba(40,28,18,0)");
                e.add_color_stop(1.0, "rgba(40,28,18,0.30)");
                ctx.set_fill_style(&e);
                ctx.fill_rect(w * 0.95, 0, w * 0.05, h);
                // feet, over the kick plate (it covers the foot's 12%):
                // soft grey smudges where shoes and carts met it
                for _ in 0..12 {
                    let (x, y) = (rnd() * w, h * (0.80 + rnd() * 0.07));
                    let (rx, ry) = (10.0 + rnd() * 26.0, 3.0 + rnd() * 6.0);
                    ctx.save();
                    ctx.translate(x, y);
                    ctx.scale(1.0, ry / rx);
                    let gr = ctx.create_radial_gradient(0, 0, 0, 0, 0, rx);
                    gr.add_color_stop(0.0, &format!("rgba(52,46,40,{:.3})", 0.10 + rnd() * 0.12));
                    gr.add_color_stop(1.0, "rgba(52,46,40,0)");
                    ctx.set_fill_style(&gr);
                    ctx.fill_rect(-rx, -rx, rx * 2.0, rx * 2.0);
                    ctx.restore();
                }
            })
        })
        .clone()
    })
}

/// A pressed-steel door frame's enamel, warm grey, as its face takes it
/// whole (u across the frame, the texture's top its head): chipped to the
/// primer and the steel at the strike (the latch jamb, at the lever's
/// height) and scuffed and rusting at the jambs' feet
fn frame_tex() -> Tex {
    FRAME.with(|c| {
        c.get_or_init(|| {
            canvas_tex_opt(256, 512, false, |ctx, w, h| {
                ctx.set_fill_style("#8a857b");
                ctx.fill_rect(0, 0, w, h);
                for _ in 0..90 {
                    let (x, y, r) = (rnd() * w, rnd() * h, 6.0 + rnd() * 26.0);
                    let gr = ctx.create_radial_gradient(x, y, 0, x, y, r);
                    gr.add_color_stop(0.0, if rnd() < 0.5 { "rgba(255,250,240,0.05)" } else { "rgba(40,36,30,0.06)" });
                    gr.add_color_stop(1.0, "rgba(0,0,0,0)");
                    ctx.set_fill_style(&gr);
                    ctx.fill_rect(x - r, y - r, r * 2.0, r * 2.0);
                }
                // chips at the strike, where the latch and keys hit it: the
                // latch jamb is u 0.95-1.0, the strike 1024 mm up of 2185
                // (0.53 down the texture); small, ragged, to the primer
                for _ in 0..7 {
                    let (x, y) = (w * (0.955 + rnd() * 0.04), h * (0.51 + rnd() * 0.04));
                    let (rx, ry) = (0.8 + rnd() * 1.8, 0.8 + rnd() * 2.6);
                    ctx.set_fill_style("rgba(70,66,60,0.35)");
                    ctx.begin_path();
                    ctx.ellipse(x, y, rx + 0.6, ry + 0.6, rnd(), 0, PI * 2.0);
                    ctx.fill();
                    ctx.set_fill_style(if rnd() < 0.5 { "rgba(112,106,98,0.9)" } else { "rgba(150,148,142,0.9)" });
                    ctx.begin_path();
                    ctx.ellipse(x, y, rx, ry, rnd(), 0, PI * 2.0);
                    ctx.fill();
                }
                // the feet: scuffed, and rust where the mop's water stood
                for x in [0.0, w * 0.95] {
                    let gr = ctx.create_linear_gradient(0, h * 0.95, 0, h);
                    gr.add_color_stop(0.0, "rgba(96,62,36,0)");
                    gr.add_color_stop(1.0, "rgba(96,62,36,0.55)");
                    ctx.set_fill_style(&gr);
                    ctx.fill_rect(x, h * 0.95, w * 0.05, h * 0.05);
                }
            })
        })
        .clone()
    })
}

/// The exit sign's face: an exit sign's own printing, ISO 7010 E002's white
/// running man through a doorway on its green, and the word EXIT; no arrow
/// (nothing points the way: `LORE.md`, *The descent*). Lit from inside.
pub fn exit_sign_tex() -> Tex {
    SIGN.with(|c| c.get_or_init(paint_exit_sign).clone())
}

fn paint_exit_sign() -> Tex {
    canvas_tex_opt(512, 256, false, |ctx, w, h| {
        ctx.set_fill_style("#1e8f57");
        ctx.fill_rect(0, 0, w, h);
        ctx.set_stroke_style("rgba(0,0,0,0.30)");
        ctx.set_line_width(4);
        ctx.stroke_rect(2, 2, w - 4.0, h - 4.0);
        // the doorway, and the man running into it
        ctx.set_fill_style("#ffffff");
        ctx.fill_rect(150, 46, 46, 164);
        ctx.set_fill_style("#1e8f57");
        ctx.fill_rect(158, 54, 30, 156);
        ctx.set_fill_style("#ffffff");
        // (each limb a bar from its joint, turned: + is clockwise, the
        // man runs to the right, into the doorway)
        let (cx, cy) = (100.0, 130.0);
        ctx.begin_path();
        ctx.arc(cx + 10.0, cy - 54.0, 16, 0, PI * 2.0);
        ctx.fill();
        for (jx, jy, rot, rw, from, len) in [
            (cx, cy, 0.25, 26.0, -32.0, 60.0),
            (cx + 4.0, cy + 22.0, -0.65, 15.0, 0.0, 60.0),
            (cx - 6.0, cy + 22.0, 0.75, 15.0, 0.0, 56.0),
            (cx + 10.0, cy - 22.0, -1.9, 12.0, 0.0, 46.0),
            (cx - 6.0, cy - 22.0, 0.95, 12.0, 0.0, 44.0),
        ] {
            ctx.save();
            ctx.translate(jx, jy);
            ctx.rotate(rot);
            ctx.fill_rect(-rw / 2.0, from, rw, len);
            ctx.restore();
        }
        ctx.set_font("bold 116px Helvetica, Arial, sans-serif");
        ctx.set_text_baseline("middle");
        ctx.fill_text("EXIT", 226, 134);
        let grad = ctx.create_linear_gradient(0, 0, 0, h);
        grad.add_color_stop(0, "rgba(255,255,255,0.08)");
        grad.add_color_stop(0.5, "rgba(255,255,255,0)");
        grad.add_color_stop(1, "rgba(0,0,0,0.12)");
        ctx.set_fill_style(&grad);
        ctx.fill_rect(0, 0, w, h);
    })
}

/// A hollow-metal door's grey enamel, as its leaf takes it whole: chipped
/// to the primer and the steel round the lock (0.92 across, 1024 mm up) and
/// along its foot, where rust has come through; a dent low on it where a
/// cart hit it; the grime of hands where it is pushed. From the side that
/// pulls (`pull`) the leaf's face is seen the other way round: the lock's
/// side on its left. (Each a painter of its own, painted once a thread:
/// cached, a painter's one paint never hangs on what was painted before.)
fn enamel_tex(pull: bool) -> Tex {
    if pull {
        ENAMEL_PULL.with(|c| c.get_or_init(|| canvas_tex_opt(256, 512, false, |ctx, w, h| paint_enamel(ctx, w, h, true))).clone())
    } else {
        ENAMEL.with(|c| c.get_or_init(|| canvas_tex_opt(256, 512, false, |ctx, w, h| paint_enamel(ctx, w, h, false))).clone())
    }
}

fn paint_enamel(ctx: &mut Canvas, w: f64, h: f64, pull: bool) {
    // (across the leaf, from the hinges on the side that pushes)
    let u = |x: f64| if pull { 1.0 - x } else { x };
    ctx.set_fill_style("#73777a");
    ctx.fill_rect(0, 0, w, h);
    for _ in 0..70 {
        let (x, y, r) = (rnd() * w, rnd() * h, 8.0 + rnd() * 30.0);
        let gr = ctx.create_radial_gradient(x, y, 0, x, y, r);
        gr.add_color_stop(0.0, if rnd() < 0.5 { "rgba(255,255,255,0.04)" } else { "rgba(20,22,24,0.05)" });
        gr.add_color_stop(1.0, "rgba(0,0,0,0)");
        ctx.set_fill_style(&gr);
        ctx.fill_rect(x - r, y - r, r * 2.0, r * 2.0);
    }
    // hands, round the lock and above it
    for (x, y, r, col) in [(0.92, 0.52, 0.18, "rgba(40,38,34,0.26)"), (0.85, 0.40, 0.14, "rgba(45,42,38,0.16)")] {
        let (x, y, r) = (u(x) * w, y * h, r * w);
        let gr = ctx.create_radial_gradient(x, y, 0, x, y, r);
        gr.add_color_stop(0.0, col);
        gr.add_color_stop(1.0, "rgba(0,0,0,0)");
        ctx.set_fill_style(&gr);
        ctx.fill_rect(x - r, y - r, r * 2.0, r * 2.0);
    }
    // chips round the lock, to the primer and the steel
    for _ in 0..14 {
        let (x, y) = (w * u(0.86 + rnd() * 0.12), h * (0.49 + rnd() * 0.07));
        let (cw, ch) = (1.2 + rnd() * 3.5, 1.2 + rnd() * 3.5);
        ctx.set_fill_style("rgba(36,36,36,0.5)");
        ctx.fill_rect(x - 0.5, y - 0.5, cw + 1.0, ch + 1.0);
        ctx.set_fill_style(if rnd() < 0.6 { "#8c5a3c" } else { "#9a9b9a" });
        ctx.fill_rect(x, y, cw, ch);
    }
    // the foot: rust come through along it, run up a little
    let gr = ctx.create_linear_gradient(0, h * 0.955, 0, h);
    gr.add_color_stop(0.0, "rgba(104,60,30,0)");
    gr.add_color_stop(0.6, "rgba(104,60,30,0.45)");
    gr.add_color_stop(1.0, "rgba(80,44,22,0.75)");
    ctx.set_fill_style(&gr);
    ctx.fill_rect(0, h * 0.955, w, h * 0.045);
    for _ in 0..22 {
        let x = rnd() * w;
        ctx.set_fill_style(format!("rgba(98,56,28,{:.3})", 0.15 + rnd() * 0.3));
        ctx.fill_rect(x, h * (0.93 + rnd() * 0.03), 1.0 + rnd() * 2.0, h * 0.04);
    }
    // a cart's dent, low on the hinge side: lit above, shaded below
    let (dx, dy, dr) = (w * u(0.3), h * 0.83, w * 0.09);
    for (oy, col) in [(-0.3, "rgba(255,255,255,0.10)"), (0.3, "rgba(0,0,0,0.16)")] {
        let gr = ctx.create_radial_gradient(dx, dy + oy * dr, 0, dx, dy + oy * dr, dr);
        gr.add_color_stop(0.0, col);
        gr.add_color_stop(1.0, "rgba(0,0,0,0)");
        ctx.set_fill_style(&gr);
        ctx.fill_rect(dx - dr, dy - dr * 1.4, dr * 2.0, dr * 2.8);
    }
}

fn veneer() -> Mat {
    phong!({ color: 0xffffff, map: &veneer_tex(), specular: 0x1c1a16, shininess: 14 })
}
fn frame_paint() -> Mat {
    phong!({ color: 0xffffff, map: &frame_tex(), specular: 0x262626, shininess: 22 })
}
/// White thermoplastic, a little yellowed
fn sign_housing() -> Mat {
    phong!({ color: 0xc8c1b1, specular: 0x222222, shininess: 16 })
}
fn satin_chrome() -> Mat {
    phong!({ color: 0xa9a9a5, specular: 0x8c8c88, shininess: 46 })
}
fn stainless() -> Mat {
    phong!({ color: 0x91908b, specular: 0x5c5c58, shininess: 34 })
}
fn sprayed_aluminium() -> Mat {
    phong!({ color: 0x9b9a96, specular: 0x3c3c3a, shininess: 24 })
}

/// The exit door in its frame on the wall at `dg`'s origin: its leaf, its
/// frame, lever, closer and kick plate, and, with `sign`, the lit sign over
/// it (the Loop's way in has none). The leaf is named `exit-door-leaf`.
pub fn exit_door(dg: &Obj, sign: bool) {
    let leaf = add(dg, &part("ExitDoor__leaf"), &veneer());
    leaf.set_name("exit-door-leaf");
    add(dg, &part("ExitDoor__frame"), &frame_paint());
    add(dg, &part("ExitDoor__lever"), &satin_chrome());
    add(dg, &part("ExitDoor__closer"), &sprayed_aluminium());
    add(dg, &part("ExitDoor__kickPlate"), &stainless());
    if sign {
        exit_door_sign(dg);
    }
}

/// The warm of the yellow rooms showing under the Works' door, the only
/// place they do (`LORE.md`, *The Building*): a light in its undercut, in
/// front of the wall's paper and behind the leaf's face, in the world at
/// the door's `spot` (named `warm-under`)
pub fn warm_under(spot: &WallSpot) -> Obj {
    let g = group();
    g.set_name("warm-under");
    g.position().copy(&spot.pos);
    g.rotation().y = spot.yaw;
    let h = UNDERCUT - 0.003;
    let m = mesh(plane_geo!(EXIT_OPEN_W - 0.02, h), basic_mat!({ color: 0xffd9a0 }));
    m.position().set(0, h / 2.0, WALL_Z + 0.0045);
    g.add(&m);
    // and spilled on the floor in front of it, fading a hand's length out
    let spill = canvas_tex_opt(64, 64, false, |ctx, w, h| {
        let gr = ctx.create_linear_gradient(0, 0, 0, h);
        gr.add_color_stop(0.0, "rgba(255,214,150,0.55)");
        gr.add_color_stop(0.35, "rgba(255,214,150,0.18)");
        gr.add_color_stop(1.0, "rgba(255,214,150,0)");
        ctx.set_fill_style(&gr);
        ctx.fill_rect(0, 0, w, h);
    });
    let depth = 0.26;
    let f = mesh(plane_geo!(EXIT_OPEN_W - 0.04, depth), basic_mat!({ map: &spill, transparent: true, depthWrite: false }));
    f.rotation().x = -PI / 2.0;
    f.position().set(0, 0.002, WALL_Z + 0.002 + depth / 2.0);
    g.add(&f);
    g
}

/// The lit exit sign over a door at `dg`'s origin (named `exit-sign`): its
/// housing on the wall a hand's width over the frame, its face lit
pub fn exit_door_sign(dg: &Obj) {
    let sign = group();
    sign.set_name("exit-sign");
    add(&sign, &part("ExitSign__housing"), &sign_housing());
    let face = mesh(plane_geo!(SIGN_FACE.0, SIGN_FACE.1), basic_mat!({ map: &exit_sign_tex() }));
    face.position().set(0, SIGN_Y, SIGN_FACE_Z);
    sign.add(&face);
    dg.add(&sign);
}

/// Which side of a steel door you see, and so what is on it
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SteelSide {
    /// the floor's, which pushes: its lever, closer and kick plate, and the
    /// STAIRS plaque that makes it a closet
    Floor,
    /// the stair's bottom door, which pushes: its push bar, closer, kick
    /// plate
    Bottom,
    /// the stair's top door, behind you, which pulls: its hinges, and
    /// nothing to take hold of
    Top,
}

/// The steel door in its frame on the wall at `dg`'s origin, from `side`
/// (its leaf is named `steel-door-leaf`). The sign over it, its lights and
/// its use are its maker's.
pub fn steel_door(dg: &Obj, side: SteelSide) {
    let enamel = phong!({ color: 0xffffff, map: &enamel_tex(side == SteelSide::Top), specular: 0x2a2a2a, shininess: 18 });
    let leaf = add(dg, &part("SteelDoor__leaf"), &enamel);
    leaf.set_name("steel-door-leaf");
    add(dg, &part("ExitDoor__frame"), &frame_paint());
    match side {
        SteelSide::Floor => {
            add(dg, &part("ExitDoor__lever"), &satin_chrome());
            add(dg, &part("ExitDoor__closer"), &sprayed_aluminium());
            add(dg, &part("ExitDoor__kickPlate"), &stainless());
            // the closet's plaque: white letters cut through black
            add(dg, &part("SteelDoor__plaque"), &phong!({ color: 0x1d1d1c, specular: 0x2a2a2a, shininess: 30 }));
            let opts = TextTexOpts {
                w: 256.0,
                h: 64.0,
                font: "bold 38px Helvetica, Arial, sans-serif".into(),
                color: "#e4e1d8".into(),
                glow: 0.0,
                ..Default::default()
            };
            let text = mesh(plane_geo!(0.19, 0.0475), lambert!({ map: text_tex("STAIRS", &opts), transparent: true }));
            text.position().set(0, PLAQUE_Y, PLAQUE_FACE_Z);
            dg.add(&text);
        }
        SteelSide::Bottom => {
            add(dg, &part("SteelDoor__pushBar"), &satin_chrome());
            add(dg, &part("ExitDoor__closer"), &sprayed_aluminium());
            add(dg, &part("ExitDoor__kickPlate"), &stainless());
        }
        SteelSide::Top => {
            add(dg, &part("SteelDoor__hinges"), &satin_chrome());
        }
    }
}
/// The STAIRS plaque's middle and the face its letters lie on
const PLAQUE_Y: f64 = 1.52;
const PLAQUE_FACE_Z: f64 = -0.015 + 0.0016 + 0.0004;

/// The exit door's parts as instanced meshes take them (the Loop's: one
/// mesh a material, `modes::loop_rooms`), the wall's face at z = 0: the
/// door's (leaf, frame, lever, closer, kick plate), and its sign's housing
/// with its material, centred on the sign's middle (its maker says how
/// high: the Loop's hangs lower, under its low ceiling)
pub fn exit_door_instanced_parts() -> (Vec<(Geo, Mat)>, (Geo, Mat)) {
    let at = |name: &str| {
        let g = part(name).duplicate();
        g.translate(0.0, 0.0, -WALL_Z);
        g
    };
    let housing = at("ExitSign__housing");
    housing.translate(0.0, -SIGN_Y, 0.0);
    let door = vec![
        (at("ExitDoor__leaf"), veneer()),
        (at("ExitDoor__frame"), frame_paint()),
        (at("ExitDoor__lever"), satin_chrome()),
        (at("ExitDoor__closer"), sprayed_aluminium()),
        (at("ExitDoor__kickPlate"), stainless()),
    ];
    (door, (housing, sign_housing()))
}

/// A lit sign face `w` x `h` in the exit sign's housing, the wall's face at
/// z = 0, centred on the sign's middle (the Loop's own face, its plain
/// EXIT: `modes::loop_rooms`)
pub fn sign_face_geo(w: f64, h: f64) -> Geo {
    let g = plane_geo!(w, h);
    g.translate(0.0, 0.0, SIGN_FACE_Z - WALL_Z);
    g
}

// ------------------------------------------------------------ hermetic door
/// The hermetic door's leaf (`HermeticDoor__*`, in its hinge's frame: x from
/// the hinge axis): its width, its thickness, the wheel's height
pub const H_LW: f64 = 1.24;
pub const H_T: f64 = 0.09;
pub const H_WHEEL_Y: f64 = 1.05;
/// The gearbox's depth on the inner face (the inner wheel turns on it)
pub const H_GEARBOX: f64 = 0.06;
/// A gallery door on the block's face (no opening behind it): its leaf's
/// middle and its outer face, out from the face; its face frame between
pub const GALLERY_LEAF_Z: f64 = 0.002 + H_T / 2.0;
pub const GALLERY_LEAF_FRONT: f64 = GALLERY_LEAF_Z + H_T / 2.0;

thread_local! {
    static HERMETIC: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() };
}

/// A shelter door's enamel, grey-green over red-brown primer, many coats
/// (`assets/src/props/hermetic_door/references.toml`, its palette), as its
/// leaf's face takes it whole (u from the hinge side, the top its head):
/// mottled where coats differ, worn through to the primer and the steel
/// along its edges and round the wheel's hub, rust run down from the hinges
/// and come up at its foot
fn hermetic_tex() -> Tex {
    HERMETIC.with(|c| {
        c.get_or_init(|| {
            canvas_tex_opt(512, 1024, false, |ctx, w, h| {
                ctx.set_fill_style("#59645c");
                ctx.fill_rect(0, 0, w, h);
                // the coats: patches a shade apart, laid on over the years
                for _ in 0..60 {
                    let (x, y, r) = (rnd() * w, rnd() * h, 20.0 + rnd() * 90.0);
                    let gr = ctx.create_radial_gradient(x, y, 0, x, y, r);
                    gr.add_color_stop(0.0, if rnd() < 0.5 { "rgba(120,132,118,0.12)" } else { "rgba(52,60,54,0.12)" });
                    gr.add_color_stop(1.0, "rgba(0,0,0,0)");
                    ctx.set_fill_style(&gr);
                    ctx.fill_rect(x - r, y - r, r * 2.0, r * 2.0);
                }
                // worn at its edges: chips to the primer, a few to the steel
                for _ in 0..260 {
                    let side = (rnd() * 4.0) as i32;
                    let t = rnd();
                    let e = rnd() * rnd() * 18.0;
                    let (x, y) = match side {
                        0 => (e, t * h),
                        1 => (w - e, t * h),
                        2 => (t * w, e),
                        _ => (t * w, h - e),
                    };
                    let r = 0.8 + rnd() * 2.6;
                    ctx.set_fill_style(if rnd() < 0.7 { "rgba(122,59,40,0.85)" } else { "rgba(140,140,134,0.85)" });
                    ctx.begin_path();
                    ctx.ellipse(x, y, r, r * (0.5 + rnd()), rnd() * 3.0, 0, PI * 2.0);
                    ctx.fill();
                }
                // round the hub (the wheel 1.05 m up of 2.06 from 0.02: 0.5
                // down the texture), where hands turned it
                let (cx, cy) = (w * 0.5, h * 0.5);
                for _ in 0..70 {
                    let a = rnd() * PI * 2.0;
                    let d = w * (0.06 + rnd() * 0.36);
                    let (x, y) = (cx + a.dcos() * d, cy + a.dsin() * d * 0.5);
                    ctx.set_fill_style(if rnd() < 0.6 { "rgba(122,59,40,0.7)" } else { "rgba(150,150,144,0.7)" });
                    ctx.fill_rect(x, y, 1.0 + rnd() * 2.5, 1.0 + rnd() * 2.0);
                }
                // rust run down from the hinges (0.45 and 1.85 m up: 0.79 and
                // 0.11 down the texture), along the hinge edge
                for hy in [0.11, 0.79] {
                    for _ in 0..9 {
                        let x = rnd() * w * 0.07;
                        let len = h * (0.04 + rnd() * 0.12);
                        let gr = ctx.create_linear_gradient(x, h * hy, x, h * hy + len);
                        gr.add_color_stop(0.0, "rgba(110,58,28,0.55)");
                        gr.add_color_stop(1.0, "rgba(110,58,28,0)");
                        ctx.set_fill_style(&gr);
                        ctx.fill_rect(x, h * hy, 1.5 + rnd() * 3.0, len);
                    }
                }
                // and come up at its foot, where the floor's wet stood
                let gr = ctx.create_linear_gradient(0, h * 0.93, 0, h);
                gr.add_color_stop(0.0, "rgba(100,56,30,0)");
                gr.add_color_stop(1.0, "rgba(100,56,30,0.6)");
                ctx.set_fill_style(&gr);
                ctx.fill_rect(0, h * 0.93, w, h * 0.07);
            })
        })
        .clone()
    })
}

/// The hermetic door's enamel (its leaf, frames, hinges, inner face)
pub fn hermetic_enamel() -> Mat {
    phong!({ color: 0xffffff, map: &hermetic_tex(), specular: 0x262826, shininess: 16 })
}
/// Its wheel, worn to the steel where hands have turned it: paler than the
/// leaf, so it reads in the dark of a cell. (Many such wheels are painted
/// red, ref 1; here red is the cleaners' mark, and a red wheel on every
/// door would say what only a marked one says: `LORE.md`, *Knocking*)
pub fn hermetic_wheel_paint() -> Mat {
    phong!({ color: 0x83877f, specular: 0x5a5c58, shininess: 34 })
}
/// Its seal's rubber
pub fn rubber() -> Mat {
    lambert!({ color: 0x1d1c1b })
}
/// A door's part as modelled (`HermeticDoor__leaf`, …), for its maker's own
/// placing (the Building's instances, a cell's swinging leaf)
pub fn part_geo(name: &str) -> Geo {
    part(name)
}

// -------------------------------------------------------------- lift doors
/// The lift's landing entrance (`LiftDoors__*`, in the lift's frame: the
/// wall's face 7.5 cm behind its origin): the frame's face, the panels'
/// middle (their faces 5 mm behind the frame's), a panel's size and middle
/// height, the opening's half width, the frame's outer half width
pub const LIFT_FACE: f64 = 0.16;
pub const LIFT_PANEL_W: f64 = 0.572;
pub const LIFT_PANEL_Z: f64 = LIFT_FACE - 0.005 - 0.016;
pub const LIFT_PANEL_Y: f64 = 0.006 + 2.094 / 2.0;
pub const LIFT_OPEN_HALF: f64 = 0.55;
pub const LIFT_FRAME_HALF: f64 = 0.67;
/// The hall call station: its middle on the wall beside the frame, and its
/// buttons (up, down) about it; a button is named `lift-button`
pub const LIFT_CALL: (f64, f64) = (0.80, 1.065);
const LIFT_WALL_Z: f64 = -0.075;
const LIFT_BUTTONS: [f64; 2] = [0.035, -0.035];

thread_local! {
    static HAIRLINE: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() };
}

/// Hairline stainless (`assets/src/props/lift_doors/references.toml`, its
/// palette), as a panel's face takes it whole: fine vertical lines, hands'
/// prints at hand height toward the middle where they meet, scuffs low
fn hairline_tex() -> Tex {
    HAIRLINE.with(|c| {
        c.get_or_init(|| {
            canvas_tex_opt(256, 512, false, |ctx, w, h| {
                ctx.set_fill_style("#8c8e92");
                ctx.fill_rect(0, 0, w, h);
                for _ in 0..420 {
                    let x = rnd() * w;
                    ctx.set_fill_style(if rnd() < 0.5 { "rgba(255,255,255,0.06)" } else { "rgba(30,32,36,0.06)" });
                    ctx.fill_rect(x, 0, 0.6 + rnd() * 1.2, h);
                }
                // the light down it, as a brushed sheet takes a ceiling's
                let g = ctx.create_linear_gradient(0, 0, 0, h);
                g.add_color_stop(0.0, "rgba(255,255,255,0.08)");
                g.add_color_stop(0.6, "rgba(0,0,0,0)");
                g.add_color_stop(1.0, "rgba(20,20,24,0.10)");
                ctx.set_fill_style(&g);
                ctx.fill_rect(0, 0, w, h);
                // prints, 1.0-1.4 m up (0.33-0.52 down the texture), at both edges
                for _ in 0..26 {
                    let x = if rnd() < 0.5 { w * rnd() * 0.25 } else { w * (1.0 - rnd() * 0.25) };
                    let y = h * (0.33 + rnd() * 0.19);
                    let r = 4.0 + rnd() * 7.0;
                    let gr = ctx.create_radial_gradient(x, y, 0, x, y, r);
                    gr.add_color_stop(0.0, "rgba(60,58,52,0.14)");
                    gr.add_color_stop(1.0, "rgba(60,58,52,0)");
                    ctx.set_fill_style(&gr);
                    ctx.fill_rect(x - r, y - r, r * 2.0, r * 2.0);
                }
                // scuffs at kick height
                for _ in 0..16 {
                    let (x, y) = (rnd() * w, h * (0.86 + rnd() * 0.1));
                    ctx.set_fill_style(format!("rgba(40,40,40,{:.3})", 0.06 + rnd() * 0.10));
                    ctx.fill_rect(x, y, 8.0 + rnd() * 30.0, 1.0 + rnd() * 2.0);
                }
            })
        })
        .clone()
    })
}

/// Hairline stainless (the panels, the frame, the call station)
pub fn hairline() -> Mat {
    phong!({ color: 0xffffff, map: &hairline_tex(), specular: 0x8e8e8c, shininess: 42 })
}

/// The lift's landing entrance on the wall at `g`'s origin: its stainless
/// frame and its aluminium sill
pub fn lift_entrance(g: &Obj) {
    add(g, &part("LiftDoors__frame"), &phong!({ color: 0x909295, specular: 0x7a7a78, shininess: 40 }));
    add(g, &part("LiftDoors__sill"), &phong!({ color: 0x7c7c78, specular: 0x4a4a48, shininess: 24 }));
}

/// One of its two panels, centred (its maker sets it)
pub fn lift_panel() -> Obj {
    mesh(part("LiftDoors__panel"), hairline())
}
/// The broken lift's panel that was forced: kicked in, its foot pushed
/// toward the shaft, hanging from its top track
pub fn lift_panel_forced() -> Obj {
    mesh(part("LiftDoors__panelForced"), hairline())
}

/// The hall call station beside the frame: its faceplate and its two
/// buttons, lit (`lit`) where the lift still comes; the buttons are named
/// `lift-button` and returned
pub fn lift_call_station(g: &Obj, lit: bool) -> Vec<Obj> {
    add(g, &part("LiftDoors__callPlate"), &phong!({ color: 0x909295, specular: 0x7a7a78, shininess: 40 }));
    let mat = if lit { lambert!({ color: 0xf0d090, emissive: 0x2a2010 }) } else { lambert!({ color: 0x3a3025 }) };
    let button = part("LiftDoors__button");
    LIFT_BUTTONS
        .iter()
        .map(|dy| {
            let b = mesh(&button, &mat);
            b.set_name("lift-button");
            b.position().set(LIFT_CALL.0, LIFT_CALL.1 + dy, LIFT_WALL_Z + 0.006);
            g.add(&b);
            b
        })
        .collect()
}
