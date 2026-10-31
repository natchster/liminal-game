//! What M. left: her camp (a bedroll against a wall, her pack, her lamp
//! still burning, empty bottles, a count of days in chalk on the wall, and
//! what she could spare: a bottle of almond water, a battery deeper down)
//! and her marks (`marks`): her sheets folded twice, and what she marked
//! the way with as she spared her paper.
//!
//! Everything here draws from the story layer's own stream, passed in; the
//! textures are painted on the native stream (`canvas_tex_opt`).
use super::{Camp, MarkKind, PlanMark, StoryPlan};
use crate::base::records::Pickup;
use crate::prelude::*;

/// How lit her camp's tile is at least while her lamp burns (the light map)
pub const CAMP_LIGHT: f64 = 0.45;
/// How long her lamp takes to gutter out once her page is taken, s
pub const GUTTER: f64 = 1.6;

impl Game {
    /// Her camp at `plan.camp`, against the wall on `plan.wall`.
    pub(crate) fn dress_camp(&mut self, plan: &StoryPlan, r: &mut SplitMix64) -> Camp {
        let (tx, tz) = plan.camp;
        let (wx, wz) = (plan.wall.0 as f64, plan.wall.1 as f64);
        let face = self
            .camp_face(plan.camp, plan.wall)
            .unwrap_or_else(|| v3(self.wx(tx) + wx * 1.6, self.floor_y_of(tx, tz), self.wz(tz) + wz * 1.6));
        let normal = v3(-wx, 0, -wz);
        // local frame: +z out from the wall, +x along it
        let camp = group();
        camp.position().copy(&face);
        camp.rotation().y = normal.x.atan2(normal.z);
        let flip = if r.next() < 0.5 { 1.0 } else { -1.0 };

        // the bedroll: a thin mattress with a blanket thrown over its foot
        let roll_mat = lambert!({ color: 0x3a4334 });
        let roll = mesh(pg("Store__bunk_mattress", box_geo!(0.72, 0.08, 1.9)), &roll_mat);
        roll.rotation().y = PI / 2.0;
        roll.position().set(0, 0.04, 0.48);
        camp.add(&roll);
        let blanket = mesh(box_geo!(0.66, 0.05, 1.05), lambert!({ color: 0x5a4632 }));
        blanket.rotation().y = PI / 2.0 + r.range(-0.08, 0.08);
        blanket.position().set(-0.38 * flip, 0.105, 0.5);
        camp.add(&blanket);
        let pillow = mesh(pg("Store__bed_pillow", box_geo!(0.46, 0.1, 0.32)), lambert!({ color: 0xa69e8e }));
        pillow.rotation().y = PI / 2.0 + r.range(-0.15, 0.15);
        pillow.position().set(0.68 * flip, 0.13, 0.46);
        camp.add(&pillow);

        // her pack, at the foot, leaning on the wall
        let pack = self.camp_pack();
        pack.position().set(-1.28 * flip, 0, 0.26);
        pack.rotation().y = r.range(-0.4, 0.4);
        pack.rotation().x = -0.12;
        camp.add(&pack);

        // her lamp, by her head, still burning
        let (lamp, light, flame) = self.camp_lamp(r.range(0.0, PI * 2.0));
        lamp.position().set(1.2 * flip, 0, 0.32);
        camp.add(&lamp);

        // empty bottles
        for (k, (lx, lz)) in [(0.25, 1.08), (0.62, 0.98)].into_iter().enumerate() {
            let b = self.camp_bottle();
            b.position().set(lx * flip + r.range(-0.1, 0.1), 0, lz + r.range(-0.08, 0.08));
            b.rotation().y = r.range(0.0, PI * 2.0) + k as f64;
            camp.add(&b);
        }

        // a count of days on the wall over the bedroll
        let days = 9 + (r.next() * 30.0) as usize;
        let tally = mesh(plane_geo!(0.7, 0.32), tally_mat(days));
        tally.position().set(-0.15 * flip, 1.05, 0.012);
        camp.add(&tally);

        // the last camp: her lamp long cold, her shoes side by side
        let last = self.level_num == super::notebook::LAST_LEVEL;
        if last {
            flame.set_visible(false);
            light.set_visible(false);
            let sole = lambert!({ color: 0x1a1210 });
            let upper = lambert!({ color: 0x3a2c22 });
            for k in [-1.0, 1.0] {
                let shoe = group();
                let s = mesh(pg("DroppedShoes__sole", box_geo!(0.28, 0.03, 0.11)), &sole);
                s.position().y = 0.015;
                shoe.add(&s);
                let u = mesh(pg("DroppedShoes__upper", box_geo!(0.24, 0.08, 0.10)), &upper);
                u.position().set(-0.02, 0.055, 0);
                shoe.add(&u);
                let t = mesh(pg("DroppedShoes__toe", box_geo!(0.08, 0.05, 0.08)), &upper);
                t.position().set(0.10, 0.045, 0);
                shoe.add(&t);
                // toes out from the wall, at the foot of the bedroll
                shoe.rotation().y = -PI / 2.0;
                shoe.position().set(-1.05 * flip + k * 0.08, 0, 0.95);
                camp.add(&shoe);
            }
        }
        camp.user().gameplay = true;
        self.world.add(&camp);

        // what she could spare: almond water, and deeper down a battery,
        // where they would be taken from a pack
        // the camp's frame in the world (a turn by its yaw: x along the wall, z out)
        let to_world = |lx: f64, lz: f64| (face.x + lx * normal.z + lz * normal.x, face.z - lx * normal.x + lz * normal.z);
        let (ax, az) = to_world(0.62 * flip, 0.85);
        self.camp_pickup(true, ax, az, r);
        if self.level_num >= 3 {
            let (bx, bz) = to_world(-0.75 * flip, 0.9);
            self.camp_pickup(false, bx, bz, r);
        }
        let at = v3(face.x + normal.x * 0.6, face.y, face.z + normal.z * 0.6);
        Camp {
            tile: plan.camp,
            wall: plan.wall,
            at,
            normal,
            lamp: Some(lamp),
            light: Some(light),
            flame: Some(flame),
            lit: !last,
            fade: if last { 0.0 } else { 1.0 },
            dimming: false,
            page: None,
            page_id: None,
            page_taken: false,
        }
    }

    /// Her page for this floor, torn from the notebook, lying on the bedroll.
    pub(crate) fn dress_page(&mut self, camp: &Camp, r: &mut SplitMix64) -> Obj {
        let tex = canvas_tex_opt(128, 166, false, |ctx, w, h| {
            ctx.set_fill_style("#e8e1cf");
            ctx.fill_rect(0, 0, w, h);
            // ruled lines and the margin
            ctx.set_stroke_style("rgba(90,110,150,0.35)");
            ctx.set_line_width(1);
            let mut y = 22.0;
            while y < h - 6.0 {
                ctx.begin_path();
                ctx.move_to(0, y);
                ctx.line_to(w, y);
                ctx.stroke();
                y += 11.0;
            }
            ctx.set_stroke_style("rgba(170,70,60,0.35)");
            ctx.begin_path();
            ctx.move_to(18, 0);
            ctx.line_to(18, h);
            ctx.stroke();
            // her hand, in pencil: a wavering line on each rule
            ctx.set_stroke_style("rgba(45,42,40,0.75)");
            ctx.set_line_width(1.6);
            for row in 0..12 {
                let y = 20.0 + row as f64 * 11.0;
                let end = if row % 5 == 4 { 60.0 } else { 112.0 - (row * 7 % 13) as f64 };
                ctx.begin_path();
                ctx.move_to(22, y);
                let mut x = 22.0;
                while x < end {
                    x += 3.0;
                    ctx.line_to(x, y - 2.0 - 1.5 * ((x * 0.9 + row as f64).sin()));
                }
                ctx.stroke();
            }
            // the torn edge along the top
            ctx.set_fill_style("rgba(20,18,14,0.9)");
            ctx.begin_path();
            ctx.move_to(0, 0);
            let mut x = 0.0;
            while x <= w {
                ctx.line_to(x, 2.0 + 2.5 * ((x * 1.7).sin() + 0.6 * (x * 0.43).cos()).abs());
                x += 4.0;
            }
            ctx.line_to(w, 0);
            ctx.close_path();
            ctx.fill();
        });
        let geo = plane_geo!(0.16, 0.21);
        geo.rotate_x(-PI / 2.0);
        let m = mesh(&geo, lambert!({ map: tex, emissive: 0x0a0907 }));
        // on the bedroll, by the pillow, turned to whoever comes to it
        let (lx, lz) = (r.range(-0.05, 0.25), 0.56 + r.range(-0.05, 0.05));
        let n = camp.normal;
        let face = v3(camp.at.x - n.x * 0.6, 0, camp.at.z - n.z * 0.6);
        m.position().set(face.x + lx * n.z + lz * n.x, camp.at.y + 0.095, face.z - lx * n.x + lz * n.z);
        m.rotation().y = n.x.atan2(n.z) + PI + r.range(-0.3, 0.3);
        m.user().gameplay = true;
        self.world.add(&m);
        m
    }

    /// A pack like the ones people leave: upright body, straps, handle, pocket.
    fn camp_pack(&self) -> Obj {
        let body_mat = lambert!({ color: 0x2a3240 });
        let strap_mat = lambert!({ color: 0x181410 });
        let bp = group();
        let body = mesh(pg("Backpacks__body", box_geo!(0.28, 0.36, 0.16)), &body_mat);
        body.position().y = 0.18;
        bp.add(&body);
        for sx in [-0.08, 0.08] {
            let strap = mesh(pg("Backpacks__strap", plane_geo!(0.04, 0.35)), &strap_mat);
            strap.rotation().y = PI;
            strap.position().set(sx, 0.18, -0.081);
            bp.add(&strap);
        }
        let handle = mesh(pg("Backpacks__handle", torus_geo!(0.04, 0.006, 4, 8, PI)), &strap_mat);
        handle.position().y = 0.36;
        bp.add(&handle);
        let pocket = mesh(pg("Backpacks__pocket", plane_geo!(0.16, 0.10)), &strap_mat);
        pocket.position().set(0, 0.12, 0.081);
        bp.add(&pocket);
        bp
    }

    /// A candle lantern a hand and a half high, lit, its flame and light
    /// flickering, guttering out over `GUTTER` s once her camp is
    /// `dimming` (then dark, and the light map without it); returns the
    /// lantern, its light and its flame.
    fn camp_lamp(&mut self, phase: f64) -> (Obj, Obj, Obj) {
        let frame_mat = lambert!({ color: 0x303030 });
        let glass_mat = basic_mat!({ color: 0xe8e0c0, transparent: true, opacity: 0.35 });
        let lantern = group();
        let base = mesh(pg("CandleLanterns__base", cyl_geo!(0.045, 0.045, 0.010, 6)), &frame_mat);
        base.position().y = 0.005;
        lantern.add(&base);
        for i in 0..4 {
            let a = (i as f64 / 4.0) * PI * 2.0 + PI / 4.0;
            let post = mesh(pg("CandleLanterns__post", cyl_geo!(0.003, 0.003, 0.14, 4)), &frame_mat);
            post.position().set(a.cos() * 0.032, 0.075, a.sin() * 0.032);
            lantern.add(&post);
        }
        for i in 0..4 {
            let a = (i as f64 / 4.0) * PI * 2.0;
            let panel = mesh(plane_geo!(0.045, 0.12), &glass_mat);
            panel.position().set(
                a.cos() * 0.032 * std::f64::consts::FRAC_1_SQRT_2,
                0.075,
                a.sin() * 0.032 * std::f64::consts::FRAC_1_SQRT_2,
            );
            panel.rotation().y = -a + PI / 2.0;
            lantern.add(&panel);
        }
        let top = mesh(pg("CandleLanterns__top", cyl_geo!(0.045, 0.045, 0.010, 6)), &frame_mat);
        top.position().y = 0.145;
        lantern.add(&top);
        let candle = mesh(pg("CandleLanterns__candle", cyl_geo!(0.012, 0.012, 0.060, 8)), lambert!({ color: 0xe2d9c6 }));
        candle.position().y = 0.045;
        candle.scale().y = 0.070 / 0.060;
        lantern.add(&candle);
        let flame = mesh(cone_geo!(0.005, 0.014, 5), basic_mat!({ color: 0xffa040 }));
        flame.position().y = 0.088;
        lantern.add(&flame);
        let handle = mesh(pg("CandleLanterns__handle", torus_geo!(0.020, 0.002, 3, 8, PI)), &frame_mat);
        handle.rotation().y = PI / 2.0;
        handle.position().y = 0.149;
        lantern.add(&handle);
        lantern.scale().set_scalar(1.7);
        // the light, in the lantern's frame (it is scaled with it)
        let light = point_light!(0xffb060, 2.2, 6.0, 1.6);
        light.position().y = 0.088;
        lantern.add(&light);
        let (f2, l2) = (flame.clone(), light.clone());
        self.animate(move |g, dt, t| {
            if !l2.visible() {
                return;
            }
            let mut fade = 1.0;
            if let Some(c) = g.story.camp.as_mut() {
                if c.dimming {
                    c.fade = (c.fade - dt / GUTTER).max(0.0);
                }
                fade = c.fade;
            }
            let f = 0.8 + 0.12 * (t * 7.0 + phase).sin() + 0.08 * (t * 11.0 + phase * 1.7).sin();
            // guttering, the flame shrinks and stutters
            let stutter = if fade < 1.0 { 0.55 + 0.45 * ((t * 23.0).sin() * (t * 7.3).cos()).abs() } else { 1.0 };
            f2.scale().set_scalar(f * fade.max(0.05));
            l2.set_intensity(2.2 * f * fade * stutter);
            if fade <= 0.0 {
                f2.set_visible(false);
                l2.set_visible(false);
                if let Some(c) = g.story.camp.as_mut() {
                    (c.lit, c.dimming) = (false, false);
                }
                g.build_light_map();
            }
        });
        (lantern, light, flame)
    }

    /// A drunk-from water bottle on its side, crushed.
    fn camp_bottle(&self) -> Obj {
        let bottle = group();
        let body = mesh(cyl_geo!(0.020, 0.024, 0.06, 8), basic_mat!({ color: 0xc0d8e0, transparent: true, opacity: 0.55 }));
        body.scale().set(1, 1, 0.5);
        body.rotation().z = PI / 2.0;
        body.position().set(0, 0.020, 0);
        bottle.add(&body);
        let cap = mesh(pg("CrushedWaterBottles__cap", cyl_geo!(0.010, 0.010, 0.008, 8)), lambert!({ color: 0x2050c0 }));
        cap.rotation().z = -PI / 2.0;
        cap.position().set(-0.034, 0.020, 0);
        bottle.add(&cap);
        bottle.scale().set_scalar(2.2);
        bottle
    }

    /// Almond water (`water`) or a battery she left, a pickup like any other.
    fn camp_pickup(&mut self, water: bool, x: f64, z: f64, r: &mut SplitMix64) {
        let pg_ = group();
        let base = self.floor_y_at(x, z) + if water { 0.15 } else { 0.11 };
        pg_.position().set(x, base, z);
        if water {
            pg_.add(&mesh(pg("Props__almond_bottle", cyl_geo!(0.09, 0.09, 0.3, 10)), lambert!({ color: 0x9fc4d8, emissive: 0x16323e })));
            let cap = mesh(pg("Props__almond_cap", cyl_geo!(0.05, 0.05, 0.06, 8)), &self.mats.metal);
            cap.position().y = 0.18;
            pg_.add(&cap);
        } else {
            pg_.add(&mesh(pg("Props__battery_body", cyl_geo!(0.06, 0.06, 0.18, 10)), lambert!({ color: 0xc79b3d, emissive: 0x2a1f08 })));
            let tip = mesh(pg("Props__battery_tip", cyl_geo!(0.025, 0.025, 0.03, 8)), lambert!({ color: 0xdad3c2 }));
            tip.position().y = 0.105;
            pg_.add(&tip);
        }
        pg_.user().gameplay = true;
        self.world.add(&pg_);
        let p = Pickup { g: pg_, taken: false, base, phase: r.range(0.0, 7.0) };
        if water {
            self.almonds.push(p);
        } else {
            self.battery_items.push(p);
        }
    }

    /// One of her marks (or a flat page), as planned, made and put where it
    /// lies; its parts shared through `kit`. Draws from `r` (the story's
    /// dressing stream) for how its bits lie.
    pub(crate) fn dress_mark(&mut self, kit: &mut MarkKit, m: &PlanMark, r: &mut SplitMix64) -> Obj {
        let fy = self.floor_y_at(m.x, m.z);
        let worn = self.level_num >= 7;
        let o = match m.kind {
            MarkKind::Sheet => {
                let (geo, mat) = kit.sheet(worn, r.next() < 0.5);
                let o = mesh(&geo, &mat);
                o.position().set(m.x, fy + 0.012, m.z);
                o.rotation().y = m.yaw;
                o
            }
            MarkKind::Flat => {
                let (geo, mat) = kit.flat();
                let o = mesh(&geo, &mat);
                o.position().set(m.x, fy + 0.012, m.z);
                o.rotation().y = m.yaw;
                o
            }
            MarkKind::Corner => {
                let (geo, mat) = kit.corner(worn);
                let o = mesh(&geo, &mat);
                o.position().set(m.x, fy + 0.008, m.z);
                o.rotation().y = m.yaw;
                o
            }
            MarkKind::Stack => {
                let what = if !self.ground.is_empty() {
                    Bits::Concrete
                } else if self.ceil_mat_key(self.t_idx(self.tx(m.x), self.tz(m.z))).starts_with("tiles:") {
                    Bits::CeilingTile
                } else {
                    Bits::Caps
                };
                let o = kit.stack(what, r);
                o.position().set(m.x, fy, m.z);
                o.rotation().y = m.yaw;
                o
            }
            MarkKind::Pencil | MarkKind::Turn | MarkKind::Chalk | MarkKind::Tape => {
                // which way along the wall she went: her arrow is turned
                // (never mirrored: her hand stays hers), a lean or an offset
                // is painted the other way round
                let face = m.normal.0.atan2(m.normal.1);
                let (ax, az) = (face.cos(), -face.sin());
                let to_right = -(m.yaw.sin()) * ax + -(m.yaw.cos()) * az >= 0.0;
                let (geo, mat) = match m.kind {
                    MarkKind::Pencil => kit.pencil(),
                    MarkKind::Turn => kit.turn(to_right),
                    MarkKind::Chalk => kit.chalk(to_right),
                    _ => kit.tape(),
                };
                let o = mesh(&geo, &mat);
                o.position().set(m.x, fy + m.y, m.z);
                o.rotation().y = face;
                if m.kind == MarkKind::Pencil && !to_right {
                    o.rotation().z = PI;
                }
                if m.kind == MarkKind::Tape {
                    o.rotation().z = r.range(-0.08, 0.08);
                }
                o
            }
        };
        o.user().gameplay = true;
        self.world.add(&o);
        o
    }

    /// Reflective tape gives your lamp back: a white glint when the beam
    /// finds it (in front of it, in the middle of the beam, nothing
    /// between), as bright as the lamp is near; grey in the room's light.
    /// One animator for every strip of her trail (`place_story`).
    pub(crate) fn tape_glints(&mut self) {
        if !self.story.trail.iter().any(|m| m.kind == MarkKind::Tape) {
            return;
        }
        self.animate(move |g, _dt, _t| {
            let lamp = g.player.flashlight_on;
            let c = g.camera.pos();
            let f = g.player.fwd();
            for m in g.story.trail.iter().filter(|m| m.kind == MarkKind::Tape) {
                let Some(o) = &m.g else { continue };
                let mut k = 0.0;
                if lamp {
                    let p = o.pos();
                    let (dx, dy, dz) = (p.x - c.x, p.y - c.y, p.z - c.z);
                    let d = (dx * dx + dy * dy + dz * dz).sqrt().max(1e-6);
                    let facing = -(dx * m.normal.0 + dz * m.normal.1) / d;
                    let centred = (dx * f.x + dy * f.y + dz * f.z) / d;
                    if d < 20.0 && facing > 0.2 && centred > 0.96 && g.los(c.x, c.z, p.x + m.normal.0 * 0.2, p.z + m.normal.1 * 0.2) {
                        k = facing.min(1.0) * ((centred - 0.96) / 0.04).min(1.0) * (1.0 - d / 20.0).max(0.0).powf(0.6);
                    }
                }
                let e = (k * 255.0) as u32;
                o.mat().m().emissive.set_hex((e << 16) | (e << 8) | e);
            }
        });
    }
}

/// What the stack of a mark is made of: what lies about where she was
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Bits {
    /// Chips of concrete (the Building's floors)
    Concrete,
    /// Broken bits of a ceiling tile (under a tiled ceiling)
    CeilingTile,
    /// Bottle caps
    Caps,
}

/// Her arrow: open, never filled, its left barb (to the left of the way it
/// points) the longer, from the tail (x0, y0) to the tip (x1, y1).
pub(crate) fn her_arrow(ctx: &mut Canvas, x0: f64, y0: f64, x1: f64, y1: f64) {
    let (dx, dy) = (x1 - x0, y1 - y0);
    let len = (dx * dx + dy * dy).sqrt().max(1e-6);
    let (ux, uy) = (dx / len, dy / len);
    // left of the way it points (the canvas's y runs down)
    let (lx, ly) = (uy, -ux);
    ctx.begin_path();
    // the shaft, a little bowed, as a hand draws it
    ctx.move_to(x0, y0);
    ctx.quadratic_curve_to((x0 + x1) / 2.0 + lx * len * 0.03, (y0 + y1) / 2.0 + ly * len * 0.03, x1, y1);
    for (side, l) in [(1.0, 0.42), (-1.0, 0.27)] {
        let (bx, by) = (x1 - ux * len * l * 0.82 + lx * side * len * l * 0.55, y1 - uy * len * l * 0.82 + ly * side * len * l * 0.55);
        ctx.move_to(x1, y1);
        ctx.line_to(bx, by);
    }
    ctx.stroke();
}

/// What her marks share, made when first needed (geometry and materials)
#[derive(Default)]
pub struct MarkKit {
    sheets: Vec<((bool, bool), (Geo, Mat))>,
    flat: Option<(Geo, Mat)>,
    corner: Vec<(bool, (Geo, Mat))>,
    pencil: Option<(Geo, Mat)>,
    turn: Vec<(bool, (Geo, Mat))>,
    chalk: Vec<(bool, (Geo, Mat))>,
    tape: Option<Geo>,
    bits: Vec<(&'static str, Geo, Mat)>,
}

/// A sheet `w` × `h` flat on the floor, one corner lifted `lift` m (a
/// curled sheet); its texture's top toward −z
fn sheet_geo(w: f64, h: f64, lift: f64) -> Geo {
    let (nu, nv) = (4usize, 3usize);
    let (mut pos, mut uv, mut idx) = (vec![], vec![], vec![]);
    for j in 0..=nv {
        for i in 0..=nu {
            let (s, t) = (i as f64 / nu as f64, j as f64 / nv as f64);
            let (x, z) = ((s - 0.5) * w, (t - 0.5) * h);
            // the corner at +x, +z curls up
            let c = ((s - 0.5).max(0.0) * 2.0 * (t - 0.5).max(0.0) * 2.0).powf(1.5);
            pos.extend_from_slice(&[x as f32, (lift * c) as f32, z as f32]);
            uv.extend_from_slice(&[s as f32, (1.0 - t) as f32]);
        }
    }
    let row = (nu + 1) as u32;
    for j in 0..nv as u32 {
        for i in 0..nu as u32 {
            let (a, b, c, d) = (j * row + i, j * row + i + 1, (j + 1) * row + i + 1, (j + 1) * row + i);
            idx.extend_from_slice(&[a, d, c, a, c, b]);
        }
    }
    let g = Geo::new();
    g.set_position(pos).set_uv(uv).set_index(idx);
    g.compute_vertex_normals();
    g
}

/// A decal on a wall: `w` × `h`, facing +z
fn wall_mat(tex: Tex, color: u32) -> Mat {
    lambert!({ map: tex, color: color, transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 })
}

impl MarkKit {
    /// Her sheet: folded twice, her arrow in pencil; deeper down greyer,
    /// stained, its pencil fainter; half of them with a corner curled up
    fn sheet(&mut self, worn: bool, curled: bool) -> (Geo, Mat) {
        if let Some((_, p)) = self.sheets.iter().find(|(k, _)| *k == (worn, curled)) {
            return p.clone();
        }
        let tex = canvas_tex_opt(128, 96, false, move |ctx, w, h| {
            ctx.set_fill_style(if worn { "#cfc8b4" } else { "#e2dbc8" });
            ctx.fill_rect(0, 0, w, h);
            // paper's grain
            for _ in 0..220 {
                let (x, y) = (rnd() * w, rnd() * h);
                ctx.set_fill_style(format!("rgba(80,70,50,{:.3})", 0.03 + rnd() * 0.05));
                ctx.fill_rect(x, y, 1.0 + rnd() * 2.0, 1.0);
            }
            if worn {
                // a damp stain from the floor
                let gr = ctx.create_radial_gradient(w * 0.7, h * 0.65, 2.0, w * 0.7, h * 0.65, w * 0.35);
                gr.add_color_stop(0.0, "rgba(110,90,55,0.28)");
                gr.add_color_stop(1.0, "rgba(110,90,55,0)");
                ctx.set_fill_style(&gr);
                ctx.fill_rect(0, 0, w, h);
            }
            // the folds: a crease across, and one down the middle
            ctx.set_stroke_style("rgba(70,60,40,0.24)");
            ctx.set_line_width(2);
            ctx.begin_path();
            ctx.move_to(0, h / 2.0);
            ctx.line_to(w, h / 2.0);
            ctx.move_to(w / 2.0, 0);
            ctx.line_to(w / 2.0, h);
            ctx.stroke();
            // her arrow, in pencil, small, up the sheet
            ctx.set_stroke_style(if worn { "rgba(60,58,55,0.5)" } else { "rgba(55,52,48,0.7)" });
            ctx.set_line_width(3.5);
            ctx.set_line_cap("round");
            ctx.set_line_join("round");
            her_arrow(ctx, w * 0.53, h * 0.74, w * 0.5, h * 0.24);
        });
        let p = (sheet_geo(0.2, 0.15, if curled { 0.018 } else { 0.0 }), lambert!({ map: tex, side: DoubleSide }));
        self.sheets.push(((worn, curled), p.clone()));
        p
    }

    /// The flat pages of a trail that is not hers: a whole page, never
    /// folded, the same heavy black arrow stamped on every one, filled and
    /// even
    fn flat(&mut self) -> (Geo, Mat) {
        if let Some(p) = &self.flat {
            return p.clone();
        }
        let tex = canvas_tex_opt(96, 128, false, |ctx, w, h| {
            ctx.set_fill_style("#e9e6dc");
            ctx.fill_rect(0, 0, w, h);
            ctx.set_fill_style("rgba(24,22,20,0.94)");
            ctx.begin_path();
            ctx.move_to(w * 0.5, h * 0.08);
            ctx.line_to(w * 0.86, h * 0.42);
            ctx.line_to(w * 0.62, h * 0.42);
            ctx.line_to(w * 0.62, h * 0.9);
            ctx.line_to(w * 0.38, h * 0.9);
            ctx.line_to(w * 0.38, h * 0.42);
            ctx.line_to(w * 0.14, h * 0.42);
            ctx.close_path();
            ctx.fill();
        });
        let geo = plane_geo!(0.21, 0.28);
        geo.rotate_x(-PI / 2.0);
        let p = (geo, lambert!({ map: tex }));
        self.flat = Some(p.clone());
        p
    }

    /// A corner torn from one of her sheets: a ragged triangle 6 cm a side,
    /// its tip the way she went (−z)
    fn corner(&mut self, worn: bool) -> (Geo, Mat) {
        if let Some((_, p)) = self.corner.iter().find(|(k, _)| *k == worn) {
            return p.clone();
        }
        // two clean edges (the sheet's corner) meeting at the tip, and the
        // torn side across, ragged
        let mut shape = Shape::new();
        shape.move_to(0.0, 0.055);
        let torn = [
            (0.026, -0.012),
            (0.019, -0.016),
            (0.012, -0.013),
            (0.006, -0.019),
            (-0.003, -0.015),
            (-0.011, -0.02),
            (-0.018, -0.014),
            (-0.024, -0.017),
        ];
        for (x, y) in torn {
            shape.line_to(x, y);
        }
        shape.line_to(0.0, 0.055);
        let geo = shape_geo!(shape);
        geo.rotate_x(-PI / 2.0);
        let p = (geo, lambert!({ color: if worn { 0xc9c1ad } else { 0xdcd5c2 }, side: DoubleSide }));
        self.corner.push((worn, p.clone()));
        p
    }

    /// Her arrow in pencil on a wall, 10 cm, pointing along it (+x)
    fn pencil(&mut self) -> (Geo, Mat) {
        if let Some(p) = &self.pencil {
            return p.clone();
        }
        let tex = canvas_tex_opt(128, 64, false, |ctx, w, h| {
            ctx.clear_rect(0, 0, w, h);
            ctx.set_stroke_style("rgba(52,52,56,0.8)");
            ctx.set_line_width(3.2);
            ctx.set_line_cap("round");
            ctx.set_line_join("round");
            her_arrow(ctx, w * 0.1, h * 0.55, w * 0.9, h * 0.47);
        });
        let p = (plane_geo!(0.12, 0.06), wall_mat(tex, 0xffffff));
        self.pencil = Some(p.clone());
        p
    }

    /// Two short pencil strokes 5 cm apart, the upper set toward the turn
    /// (`to_right`: toward +x)
    fn turn(&mut self, to_right: bool) -> (Geo, Mat) {
        if let Some((_, p)) = self.turn.iter().find(|(k, _)| *k == to_right) {
            return p.clone();
        }
        let tex = canvas_tex_opt(64, 64, false, move |ctx, w, h| {
            ctx.clear_rect(0, 0, w, h);
            ctx.set_stroke_style("rgba(48,48,52,0.85)");
            ctx.set_line_width(4.5);
            ctx.set_line_cap("round");
            let s = if to_right { 1.0 } else { -1.0 };
            ctx.begin_path();
            // the upper, set toward the turn
            ctx.move_to(w * (0.5 + s * 0.13) - w * 0.25, h * 0.28);
            ctx.line_to(w * (0.5 + s * 0.13) + w * 0.25, h * 0.25);
            // the lower
            ctx.move_to(w * 0.5 - w * 0.25, h * 0.72);
            ctx.line_to(w * 0.5 + w * 0.25, h * 0.71);
            ctx.stroke();
        });
        let p = (plane_geo!(0.09, 0.075), wall_mat(tex, 0xffffff));
        self.turn.push((to_right, p.clone()));
        p
    }

    /// A chalk stroke 8 cm long leaning on (`to_right`: toward +x)
    fn chalk(&mut self, to_right: bool) -> (Geo, Mat) {
        if let Some((_, p)) = self.chalk.iter().find(|(k, _)| *k == to_right) {
            return p.clone();
        }
        let tex = canvas_tex_opt(64, 128, false, move |ctx, w, h| {
            ctx.clear_rect(0, 0, w, h);
            let s = if to_right { 1.0 } else { -1.0 };
            // chalk on a rough wall: a stroke broken where the wall is low,
            // heavier where it was pressed at the start
            for k in 0..220 {
                let t = k as f64 / 220.0;
                let x = w * 0.5 + s * (t - 0.5) * w * 0.5 + (rnd() - 0.5) * 3.0;
                let y = h * (0.9 - t * 0.8) + (rnd() - 0.5) * 2.0;
                if rnd() < 0.86 {
                    let a = (0.95 - t * 0.35) * (0.6 + rnd() * 0.4);
                    ctx.set_fill_style(format!("rgba(232,228,214,{a:.3})"));
                    ctx.fill_rect(x - 4.5, y - 2.0, 9.0, 4.0);
                }
            }
        });
        let p = (plane_geo!(0.07, 0.12), wall_mat(tex, 0xffffff));
        self.chalk.push((to_right, p.clone()));
        p
    }

    /// A strip of reflective tape, 8 × 2.5 cm: its own material (each glints
    /// on its own)
    fn tape(&mut self) -> (Geo, Mat) {
        let geo = self.tape.get_or_insert_with(|| plane_geo!(0.08, 0.025)).clone();
        (geo, lambert!({ color: 0x8c8c88, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 }))
    }

    /// Three bits of `what`: a large one, a small one on it, a third beside
    /// them the way she went (−z)
    fn stack(&mut self, what: Bits, r: &mut SplitMix64) -> Obj {
        let key = match what {
            Bits::Concrete => "concrete",
            Bits::CeilingTile => "tile",
            Bits::Caps => "caps",
        };
        if !self.bits.iter().any(|b| b.0 == key) {
            let (geo, mat) = match what {
                Bits::Concrete => (dodeca_geo!(1.0), lambert!({ color: 0x8b867c })),
                Bits::CeilingTile => (box_geo!(1.0, 1.0, 1.0), lambert!({ color: 0xd2ccbc })),
                Bits::Caps => (cyl_geo!(1.0, 1.0, 1.0, 10), lambert!({ color: 0x2c4f9a })),
            };
            self.bits.push((key, geo, mat));
        }
        let (_, geo, mat) = self.bits.iter().find(|b| b.0 == key).unwrap();
        let g = group();
        // (size across, height) of the large, the small and the third
        let sizes: [(f64, f64); 3] = match what {
            Bits::Concrete => [(0.07, 0.045), (0.04, 0.028), (0.045, 0.03)],
            Bits::CeilingTile => [(0.09, 0.016), (0.05, 0.016), (0.06, 0.016)],
            Bits::Caps => [(0.03, 0.012), (0.03, 0.012), (0.03, 0.012)],
        };
        let mut top = 0.0;
        for (k, &(w, h)) in sizes.iter().enumerate() {
            let o = mesh(geo, mat);
            let s = 1.0 + r.range(-0.15, 0.15);
            match what {
                Bits::Concrete => o.scale().set(w / 2.0 * s, h / 2.0, w / 2.0 * (2.0 - s)),
                Bits::CeilingTile => o.scale().set(w * s, h, w * 0.7 * (2.0 - s)),
                Bits::Caps => o.scale().set(w / 2.0, h, w / 2.0),
            };
            o.rotation().y = r.range(0.0, PI * 2.0);
            let (x, z, y) = match k {
                0 => (0.0, 0.0, h / 2.0),
                1 => (r.range(-0.006, 0.006), r.range(-0.006, 0.006), top + h / 2.0),
                _ => (r.range(-0.015, 0.015), -0.13 - r.range(0.0, 0.03), h / 2.0),
            };
            if k == 0 {
                top = h;
            }
            o.position().set(x, y, z);
            g.add(&o);
        }
        g
    }
}

/// Chalk tallies on a wall: `days` strokes in fives.
fn tally_mat(days: usize) -> Mat {
    let tex = canvas_tex_opt(256, 116, false, move |ctx, _w, h| {
        ctx.set_fill_style("rgba(0,0,0,0)");
        ctx.fill_rect(0, 0, 256, h);
        ctx.set_stroke_style("rgba(225,220,205,0.75)");
        ctx.set_line_width(3);
        ctx.set_line_cap("round");
        for i in 0..days {
            let (grp, k) = (i / 5, i % 5);
            let (row, col) = (grp / 6, grp % 6);
            let x0 = 10.0 + col as f64 * 40.0;
            let y0 = 14.0 + row as f64 * 50.0;
            ctx.begin_path();
            if k < 4 {
                let x = x0 + k as f64 * 7.0;
                ctx.move_to(x, y0);
                ctx.line_to(x + 1.5, y0 + 34.0);
            } else {
                ctx.move_to(x0 - 4.0, y0 + 28.0);
                ctx.line_to(x0 + 26.0, y0 + 6.0);
            }
            ctx.stroke();
        }
    });
    basic_mat!({ map: tex, transparent: true, depthWrite: false, opacity: 0.8 })
}
