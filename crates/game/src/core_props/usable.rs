//! Things a hand can use (`docs/design/PROPS.md` 3): the props on every floor that
//! answer a verb, registered once the props have settled
//! (`usable_settled`; an interactable pins the wall faces near it while
//! settling, so none is registered before). A box is opened, a bin looked
//! into, a book stack gives its pages, a TV is switched off and on, a
//! locker whose padlock hangs open is opened, a dead lift's button is
//! pressed.
//!
//! What each holds is fixed when the floor is built, from a stream of its
//! own (`USE_SALT`), never rolled when it is used; nothing says what a
//! thing does, only what is found (in the words of a pickup). Its sound
//! comes from it (`play_from`), and a creature hears it as it hears the
//! machines: through its memory of where a sound was, never by being set
//! to hunt. None on the Building's floors, the Lobby, or a floor with rules
//! of its own.
use crate::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

/// The verbs' stream (`SplitMix64` with the level's seed)
pub const USE_SALT: u64 = 0x05E5_7A1C_0DD5_11FE;
/// How near a hand must be (m, from the eye; `research/props-interactions.md`:
/// about 1.2 m, so clutter does not ring from across a room)
const REACH: f64 = 1.2;
/// How loud a verb is to what hears it (its awareness; a creature stops to
/// listen from `SUSPECT`, 0.3): the tape, a locker's door and a set coming
/// on stop one near; a bin's rustle and a page torn out reach only one
/// already listening
const LOUD: f64 = 0.45;
const QUIET: f64 = 0.2;
/// From this level the books have taken the damp: a page or two comes away
const DAMP_BOOKS: i32 = 9;
/// A box holds a few sheets this often; a bin one or two
const BOX_PAPER: f64 = 0.25;
const BIN_PAPER: f64 = 0.3;
/// A shut locker's padlock hangs open this often; one holds almond water this often
const LOCKER_OPEN: f64 = 0.4;
const LOCKER_WATER: f64 = 0.15;
/// One dead lift in this many has a car go by behind its doors
const LIFT_PASSES: f64 = 1.0 / 3.0;

impl Game {
    /// Give the floor's usable props their verbs (after settling)
    pub fn usable_settled(&mut self) {
        // a lit set hisses (close by: the CRTs' hiss), on any floor
        if let Some(tv) = self.tv_data.as_ref().filter(|t| t.on) {
            self.pp.crt_positions.push(tv.world);
        }
        if self.level_num <= 4 || self.level_scp.is_some() {
            return;
        }
        let mut r = SplitMix64::new(self.current_seed, USE_SALT);
        for o in self.world.children() {
            let kind = o.user().s.get("kind").cloned();
            match kind.as_deref() {
                Some("cardboard_stack") => self.usable_box(&o, &mut r),
                Some("trash_can") => self.usable_bin(&o, &mut r),
                Some("book_stack") => self.usable_books(&o, &mut r),
                Some("locker") => self.usable_lockers(&o, &mut r),
                Some("elevators") if o.user().s.contains_key("dead_lift") => self.usable_dead_lift(&o, &mut r),
                _ => {}
            }
        }
        if self.tv_data.is_some() {
            self.usable_tv();
        }
    }

    /// Where a hand reaches a prop from: over it at chest height
    fn use_point(o: &Obj, h: f64) -> Vec3 {
        let p = o.pos();
        v3(p.x, h.max(0.6), p.z)
    }

    /// The top box of a stack opens: its tape slit, its flaps out, packing
    /// paper in it; a quarter of them hold a few sheets as well
    fn usable_box(&mut self, o: &Obj, r: &mut SplitMix64) {
        let Some(top) = o.children().last().cloned() else { return };
        let sheets = if r.next() < BOX_PAPER { 2 + (r.next() * 3.0) as i32 } else { 0 };
        let opened = Rc::new(Cell::new(false));
        let (en, used) = (opened.clone(), opened.clone());
        let mut bb = Box3::empty();
        o.update_world_matrix(true, true);
        bb.set_from_object(&top);
        let at = v3((bb.min.x + bb.max.x) / 2.0, bb.max.y, (bb.min.z + bb.max.z) / 2.0);
        self.interact(
            Self::use_point(o, at.y + 0.2),
            REACH,
            "[E] OPEN THE BOX",
            move |_g| !en.get(),
            move |g| {
                used.set(true);
                open_box(&top);
                g.prop_sound(at, PropSound::TapeRip);
                g.prop_noise(at, 6.0, LOUD);
                if sheets > 0 {
                    g.player.papers += sheets;
                    g.show_msg(&format!("PAPER — {sheets} SHEETS"), 1.6);
                }
            },
        );
    }

    /// A bin is looked into: litter, and now and then a sheet worth keeping
    fn usable_bin(&mut self, o: &Obj, r: &mut SplitMix64) {
        let sheets = if r.next() < BIN_PAPER { 1 + (r.next() * 2.0) as i32 } else { 0 };
        let looked = Rc::new(Cell::new(false));
        let (en, used) = (looked.clone(), looked.clone());
        let at = o.pos() + v3(0.0, 0.6, 0.0);
        self.interact(
            Self::use_point(o, 1.0),
            REACH,
            "[E] LOOK INSIDE",
            move |_g| !en.get(),
            move |g| {
                used.set(true);
                g.prop_sound(at, PropSound::Rustle);
                g.prop_noise(at, 4.0, QUIET);
                if sheets > 0 {
                    g.player.papers += sheets;
                    g.show_msg(&format!("PAPER — {sheets} {}", if sheets == 1 { "SHEET" } else { "SHEETS" }), 1.6);
                }
            },
        );
    }

    /// A book stack gives its pages to write on: once, a few, torn out of
    /// the top book; deeper down, where the damp has got into them, a page
    /// or two (nothing is left on the floor: a sheet there is the Reader's)
    fn usable_books(&mut self, o: &Obj, r: &mut SplitMix64) {
        let k = r.next();
        let sheets = if self.level_num >= DAMP_BOOKS { 1 + (k * 2.0) as i32 } else { 3 + (k * 3.0) as i32 };
        let torn = Rc::new(Cell::new(false));
        let (en, used) = (torn.clone(), torn.clone());
        let at = o.pos() + v3(0.0, 0.2, 0.0);
        self.interact(
            Self::use_point(o, 0.9),
            REACH,
            "[E] TEAR OUT PAGES",
            move |_g| !en.get(),
            move |g| {
                used.set(true);
                g.prop_sound(at, PropSound::Tear);
                g.prop_noise(at, 4.0, QUIET);
                g.player.papers += sheets;
                g.show_msg(&format!("PAPER — {sheets} {}", if sheets == 1 { "SHEET" } else { "SHEETS" }), 1.6);
            },
        );
    }

    /// A shut locker whose padlock hangs open swings open: a jacket on its
    /// hook, a jumper folded on its floor, now and then a bottle of almond
    /// water in the jacket's pocket
    fn usable_lockers(&mut self, o: &Obj, r: &mut SplitMix64) {
        let units = o.user().get("units") as usize;
        for i in 0..units {
            let unit = i as i32;
            let (door, face, x) = {
                let u = o.user();
                let Some(d) = u.o.get(&format!("door{i}")).cloned() else { continue };
                (d, u.get(&format!("face{i}")) as usize, u.get(&format!("x{i}")))
            };
            if r.next() >= LOCKER_OPEN {
                continue;
            }
            let water = r.next() < LOCKER_WATER;
            // the tell: its shackle lifted out of the body and turned
            let kids = o.children();
            let shackle =
                kids.iter().skip(face).find(|c| (c.pos().y - 0.83).abs() < 1e-6 && (c.pos().x - (x - 0.03)).abs() < 1e-6).cloned();
            if let Some(s) = &shackle {
                s.position().y = 0.86;
                s.rotation().y = 0.9;
            }
            let opened = Rc::new(Cell::new(false));
            let (en, used) = (opened.clone(), opened.clone());
            let locker = o.clone();
            o.update_world_matrix(true, true);
            let front = o.local_to_world(&v3(x, 1.2, 0.6));
            let at = o.local_to_world(&v3(x, 0.9, 0.3));
            self.interact(
                front,
                REACH,
                "[E] OPEN THE LOCKER",
                move |_g| !en.get(),
                move |g| {
                    used.set(true);
                    // its door flattened to a leaf, its face riding on it, swung about its hinge
                    door.scale().z = 0.08;
                    door.position().z = 0.23;
                    let lf = group();
                    lf.position().set(x - 0.21, 0.0, 0.25);
                    locker.add(&lf);
                    let parts: Vec<Obj> = locker.children()[face..]
                        .iter()
                        .filter(|c| {
                            let p = c.pos();
                            (p.x - x).abs() < 0.22 && p.z > 0.2 && !c.ptr_eq(&lf)
                        })
                        .cloned()
                        .collect();
                    for c in &parts {
                        crate::core_props::builders::attach(&lf, c);
                    }
                    let mut t = 0.0;
                    g.animate(move |_g, dt, _t| {
                        if t >= 0.7 {
                            return;
                        }
                        t = (t + dt).min(0.7);
                        let k = t / 0.7;
                        lf.rotation().y = -1.9 * (1.0 - (1.0 - k) * (1.0 - k));
                    });
                    // its sides, and what is in it: a jacket on its hook, a jumper folded on its floor
                    for c in crate::core_props::builders::locker_sides(g, x)
                        .into_iter()
                        .chain(crate::core_props::builders::locker_things(x, unit, true))
                    {
                        locker.add(&c);
                    }
                    g.prop_sound(at, PropSound::Locker);
                    g.prop_noise(at, 8.0, LOUD);
                    if water {
                        g.player.bottles += 1;
                        g.show_msg("ALMOND WATER — IN THE JACKET'S POCKET", 1.8);
                    }
                },
            );
        }
    }

    /// A dead lift's button is pressed: it lights, weakly, and goes out;
    /// behind some, a car is heard going by a few seconds later, and it
    /// never stops
    fn usable_dead_lift(&mut self, o: &Obj, r: &mut SplitMix64) {
        let passes = r.next() < LIFT_PASSES;
        o.update_world_matrix(true, true);
        let buttons: Vec<Obj> = o
            .children()
            .into_iter()
            .filter(|c| c.is_mesh() && (c.pos().x - 0.88).abs() < 0.02 && c.pos().y > 1.3 && c.pos().y < 1.5 && c.pos().z > 0.09)
            .collect();
        let pos = o.local_to_world(&v3(0.88, 1.4, 0.7));
        let at = o.local_to_world(&v3(0.88, 1.4, 0.1));
        let shaft = o.local_to_world(&v3(0.0, 1.2, -0.3));
        self.interact(
            pos,
            REACH,
            "[E] PRESS THE BUTTON",
            |_g| true,
            move |g| {
                g.prop_sound(at, PropSound::Button);
                for b in &buttons {
                    b.set_mat(&lambert!({ color: 0x8a7040, emissive: 0x3a2808 }));
                    let b = b.clone();
                    g.schedule(900.0, move |_g| {
                        b.set_mat(&lambert!({ color: 0x3a3025 }));
                    });
                }
                if passes {
                    g.schedule(4000.0, move |g| {
                        g.prop_sound(shaft, PropSound::CarPasses);
                    });
                    // your asides: once it has gone by (4 s on, 2.6 s of rumble)
                    g.aside_later(6900.0, crate::audio::asides::Use::Uncanny, "lift");
                }
            },
        );
    }

    /// The CRT is switched off (its picture falls to a dot and goes, its
    /// light with it) and on again (the static and its light come back)
    fn usable_tv(&mut self) {
        let Some(tv) = self.tv_data.clone() else { return };
        let at = tv.world;
        let pos = v3(at.x + tv.normal.x * 0.7, 1.1, at.z + tv.normal.z * 0.7);
        self.interact(
            pos,
            REACH,
            "[E] SWITCH IT OFF",
            |g| g.tv_data.as_ref().is_some_and(|t| t.on),
            move |g| {
                g.tv_switch(false);
                g.prop_sound(at, PropSound::CrtOff);
            },
        );
        self.interact(
            pos,
            REACH,
            "[E] SWITCH IT ON",
            |g| g.tv_data.as_ref().is_some_and(|t| !t.on),
            move |g| {
                g.tv_switch(true);
                g.prop_sound(at, PropSound::CrtOn);
                g.prop_noise(at, 8.0, LOUD);
            },
        );
    }

    /// The TV on or off: its screen's static or a dead tube, its light
    pub fn tv_switch(&mut self, on: bool) {
        let Some(tv) = self.tv_data.as_mut() else { return };
        tv.on = on;
        // its hiss with it
        let at = tv.world;
        self.pp.crt_positions.retain(|p| p.distance_to_squared(&at) > 1e-6);
        if on {
            self.pp.crt_positions.push(at);
        }
        let Some(tv) = self.tv_data.as_mut() else { return };
        let screen = tv.screen.clone();
        let light = tv.light.clone();
        if on {
            screen.set_mat(&basic_mat!({ map: &self.tex.static_tex }));
        } else {
            light.set_intensity(0.0);
            // the tube's dark glass, a bright dot at its middle that shrinks and goes
            screen.set_mat(&lambert!({ color: 0x1c1f1e }));
            let dot = mesh(circle_geo!(0.03, 12), basic_mat!({ color: 0xd8e4f0, transparent: true }));
            dot.position().z = 0.002;
            screen.add(&dot);
            let mut t = 0.0;
            self.animate(move |_g, dt, _t| {
                if t > 0.6 {
                    return;
                }
                t += dt;
                let k = (1.0 - t / 0.6).max(0.0);
                dot.set_scale(k, k, 1.0);
                dot.mat().set_opacity(k);
                if k <= 0.0 {
                    dot.set_visible(false);
                }
            });
        }
    }

    /// A small sound a prop makes where it is (played only within earshot:
    /// a sound draws from the frame stream)
    fn prop_sound(&mut self, at: Vec3, what: PropSound) {
        let pp = self.player.pos;
        if hypot2(pp.x - at.x, pp.z - at.z) > 30.0 {
            return;
        }
        self.play_from(at.x, at.y, at.z, 1.5, 1.0, 0.7, move |g| match what {
            PropSound::TapeRip => {
                g.noise_burst(0.32, 3600.0, 1.4, 0.07, "highpass", Some(1800.0), 0.0, 0.0);
                g.noise_burst(0.18, 900.0, 0.8, 0.05, "bandpass", None, 0.3, 0.0);
            }
            PropSound::Rustle => {
                g.noise_burst(0.22, 2600.0, 0.9, 0.05, "bandpass", None, 0.0, 0.0);
                g.noise_burst(0.16, 1800.0, 0.9, 0.04, "bandpass", None, 0.25, 0.0);
            }
            PropSound::Tear => {
                g.noise_burst(0.4, 4200.0, 1.2, 0.06, "highpass", Some(2400.0), 0.0, 0.0);
            }
            PropSound::CrtOff => {
                g.blip(15600.0, 0.5, "sine", 0.012, Some(9000.0), 0.0, 0.0);
                g.noise_burst(0.08, 400.0, 1.0, 0.05, "lowpass", None, 0.0, 0.0);
            }
            PropSound::Locker => {
                g.noise_burst(0.05, 1800.0, 2.0, 0.06, "bandpass", None, 0.0, 0.0);
                g.blip(420.0, 0.5, "sawtooth", 0.006, Some(260.0), 0.04, 0.0);
                g.noise_burst(0.3, 600.0, 1.0, 0.04, "lowpass", None, 0.1, 0.0);
            }
            PropSound::Button => {
                g.noise_burst(0.02, 3000.0, 2.0, 0.05, "bandpass", None, 0.0, 0.0);
            }
            PropSound::CarPasses => {
                // a car going by in the shaft: a low rumble rising and falling
                g.noise_burst(2.6, 120.0, 0.7, 0.09, "lowpass", Some(260.0), 0.0, 0.0);
                g.noise_burst(1.4, 260.0, 0.7, 0.06, "lowpass", Some(90.0), 1.2, 0.0);
                g.blip(55.0, 2.4, "sawtooth", 0.01, Some(70.0), 0.2, 0.0);
            }
            PropSound::CrtOn => {
                g.noise_burst(0.12, 300.0, 1.0, 0.07, "lowpass", None, 0.0, 0.0);
                g.blip(15600.0, 0.8, "sine", 0.01, None, 0.05, 0.0);
            }
        });
    }

    /// A sound a creature can hear where a prop is, as it hears a machine:
    /// its memory of where it was and how sure it is, within `reach` along
    /// the way (times its hearing)
    pub fn prop_noise(&mut self, at: Vec3, reach: f64, loud: f64) {
        let (gx, gz) = self.open_point(at.x, at.z);
        let now = self.perf_t;
        for ei in 0..self.entities.len() {
            let Some(c) = self.creature(ei) else { continue };
            if c.data.hearing <= 0.0 {
                continue;
            }
            let p = self.entities[ei].pos();
            let straight = hypot2(p.x - at.x, p.z - at.z);
            let d = self.path_distance(p.x, p.z, straight);
            if d > reach * c.data.hearing {
                continue;
            }
            if let Some(c) = self.creature_mut(ei) {
                c.brain.mem.heard = Some((gx, gz, now));
                c.brain.mem.awareness = c.brain.mem.awareness.max(loud);
            }
        }
        self.director_note("prop");
    }
}

/// What a prop sounds like being used
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PropSound {
    TapeRip,
    Rustle,
    Tear,
    CrtOff,
    CrtOn,
    Locker,
    Button,
    CarPasses,
}

/// A box's tape slit and its flaps folded out, crumpled packing paper
/// showing in it
fn open_box(top: &Obj) {
    let mut bb = Box3::empty();
    top.update_world_matrix(true, true);
    bb.set_from_object(top);
    let s = top.scl();
    // its size in its own frame (the box is a part fitted to its size)
    let sz = ((bb.max.x - bb.min.x).min(bb.max.z - bb.min.z)).clamp(0.3, 0.6) / s.x.max(1e-3);
    // the tape over its top goes (it was slit)
    for c in top.children() {
        let p = c.pos();
        if (p.y - 0.163).abs() < 0.01 && c.is_mesh() {
            c.set_visible(false);
        }
    }
    let flap_mat = lambert!({ color: 0x8b6a42 });
    for k in 0..4 {
        let a = k as f64 * PI / 2.0;
        let flap = mesh(box_geo!(sz * 0.96, 0.005, sz * 0.48), &flap_mat);
        let hinge = group();
        hinge.rotation().y = a;
        hinge.position().y = 0.16;
        // hinged at its edge, folded out and down past the side
        let f = group();
        f.position().z = sz / 2.0;
        f.rotation().x = 1.9 + 0.15 * (k as f64 - 1.5);
        flap.position().z = sz * 0.24;
        f.add(&flap);
        hinge.add(&f);
        top.add(&hinge);
    }
    // balled packing paper (the crumpled sheets of `place_scrap_papers`)
    let paper = lambert!({ color: 0xc9c2b0 });
    for k in 0..3 {
        let part = if k == 1 { "Real__paperBall" } else { "Real__paperWad" };
        let geo = crate::assets::props::prop_part_as_modelled(part).unwrap_or_else(|| sphere_geo!(0.04, 7, 5));
        let ball = mesh(geo, &paper);
        ball.position().set((k as f64 - 1.0) * sz * 0.22, 0.12, (k % 2) as f64 * 0.06 - 0.03);
        ball.rotation().set(k as f64 * 1.3, k as f64 * 2.1, 0.4);
        ball.scale().set(1.6, 1.6, 1.6);
        top.add(&ball);
    }
}
