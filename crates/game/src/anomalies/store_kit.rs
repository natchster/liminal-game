//! The Showroom (`anomalies::store_kit`), one of the run's anomaly levels (`level::scp` 'store').
//! Inspired by SCP-3008 "A Perfectly Normal, Regular Old IKEA" by Mortos
//! (CC BY-SA 3.0, https://scp-wiki.wikidot.com/scp-3008); see docs/design/LORE.md.
//! No brand, no names: a flat-pack furniture store that never closes.
//!
//! This module is the part that does not need the game: the opening hours and
//! the room-set furniture kits. The game places them (`place_store`), runs
//! the hours (`update_store`) and drives the staff.
//!
//! Every piece of furniture is built from primitives wrapped in the
//! prop-part wrapper (`pg("Store__…", geo)`), so the Blender parts replace
//! them, fitted to the same bounds.

use engine::prelude::*;
use std::f64::consts::PI;
use std::rc::Rc;

// ---------------------------------------------------------------- hours ---
/// `STORE_HOURS`: a day is long enough to cross a good part of the floor; the
/// night is short and loud. The first day is a little shorter than the rest.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StoreHours {
    pub first_day: f64,
    pub day: f64,
    pub warn: f64,
    pub night: f64,
}
pub const STORE_HOURS: StoreHours = StoreHours { first_day: 105.0, day: 140.0, warn: 14.0, night: 48.0 };

/// The part of the store's cycle: open, the closing warning, or night.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Day,
    Warn,
    Night,
}
impl Phase {
    pub fn as_str(&self) -> &'static str {
        match self {
            Phase::Day => "day",
            Phase::Warn => "warn",
            Phase::Night => "night",
        }
    }
}

/// `store_phase(t)`'s result.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StorePhase {
    pub phase: Phase,
    pub left: f64,
    pub into: f64,
    pub cycle: i32,
    pub len: f64,
}

/// Where the store is in its cycle `t` seconds after the level began.
pub fn store_phase(t: f64, h: &StoreHours) -> StorePhase {
    let mut left = t;
    let mut day = h.first_day;
    let mut n = 0;
    loop {
        if left < day {
            let warn = left >= day - h.warn;
            return StorePhase { phase: if warn { Phase::Warn } else { Phase::Day }, left: day - left, into: left, cycle: n, len: day };
        }
        left -= day;
        if left < h.night {
            return StorePhase { phase: Phase::Night, left: h.night - left, into: left, cycle: n, len: h.night };
        }
        left -= h.night;
        day = h.day;
        n += 1;
    }
}
/// `store_phase(t)` with the default hours.
pub fn store_phase_default(t: f64) -> StorePhase {
    store_phase(t, &STORE_HOURS)
}

/// The clock the store boards show, opening at 10:00 and
/// closing at 21:00, the day squeezed into the cycle. Night reads CLOSED (None).
pub fn store_clock(p: &StorePhase) -> Option<String> {
    if p.phase == Phase::Night {
        return None;
    }
    let f = 1f64.min(p.into / p.len);
    let mins = (10.0 * 60.0 + f * 11.0 * 60.0).floor() as i64;
    Some(format!("{:02}:{:02}", mins / 60, mins % 60))
}

// -------------------------------------------------------------- names -----
const SYL_A: [&str; 15] = ["BJ", "GR", "H", "KL", "L", "M", "N", "R", "SK", "ST", "SV", "T", "V", "FL", "DR"];
const SYL_V: [&str; 9] = ["A", "Ä", "E", "I", "O", "Ö", "U", "Å", "Y"];
const SYL_B: [&str; 12] = ["RK", "LL", "NS", "ST", "MMA", "GEN", "RA", "NDE", "KT", "SA", "LBY", "RP"];
/// A made-up product name.
pub fn product_name(rng: &mut dyn FnMut() -> f64) -> String {
    let mut p = |a: &[&'static str]| -> &'static str { a[(rng() * a.len() as f64).floor() as usize] };
    let mut s = String::new();
    s.push_str(p(&SYL_A));
    s.push_str(p(&SYL_V));
    s.push_str(p(&SYL_B));
    if rng() < 0.4 {
        s.push_str(SYL_V[(rng() * SYL_V.len() as f64).floor() as usize]);
    }
    s
}

// ------------------------------------------------------------- the kit ----
/// A price tag's text: the product name, what it is, and the price.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TagText {
    pub name: String,
    pub what: String,
    pub price: String,
}

/// The hiding spot of a group that can hide a crouching player (kept in its user data).
#[derive(Clone, Debug, PartialEq)]
pub struct HideSpot {
    pub eye: f64,
    pub r: f64,
    pub kind: String,
    pub front: Option<f64>,
}
/// Store a hiding spot in a group's user data (`hide = true` plus the keys
/// `hide_eye`, `hide_r`, the optional `hide_front` and `s["hide_kind"]`).
pub fn set_hide_spot(o: &Obj, h: &HideSpot) {
    let mut u = o.user();
    u.hide = true;
    u.set("hide_eye", h.eye);
    u.set("hide_r", h.r);
    if let Some(f) = h.front {
        u.set("hide_front", f);
    }
    u.s.insert("hide_kind".into(), h.kind.clone());
}
/// Read a group's hiding spot back (None when the group hides nobody).
pub fn hide_spot(o: &Obj) -> Option<HideSpot> {
    let n = o.r();
    let u = &n.user;
    if !u.hide {
        return None;
    }
    Some(HideSpot {
        eye: u.get("hide_eye"),
        r: u.get("hide_r"),
        kind: u.s.get("hide_kind").cloned().unwrap_or_default(),
        front: u.f.get("hide_front").copied(),
    })
}

/// What the kit builds with: the prop-part wrapper, a material per colour,
/// the RNG and a price-tag material maker.
pub struct StoreKitDeps {
    /// the prop-part wrapper; the game's is `placers::kit::pg`
    pub pg: Rc<dyn Fn(&str, Geo) -> Geo>,
    /// a cached Lambert material for a colour
    pub mat: Box<dyn FnMut(u32) -> Mat>,
    /// `rnd()` in the game
    pub rng: Box<dyn FnMut() -> f64>,
    /// a price-tag material with the given text drawn on it
    pub tag: Box<dyn FnMut(&TagText) -> Mat>,
}

const WOOD: [u32; 5] = [0xc9a97a, 0xb08858, 0xe0cfb0, 0x7a5a3a, 0xf0ece4];
const FABRIC: [u32; 7] = [0x6d7a86, 0x8a7a66, 0x4f5a4a, 0xa89a88, 0x3f4450, 0x9a5a48, 0xc8c0b0];

/// The furniture builders. Each returns a group standing on y = 0, its front
/// facing +z. Groups that can hide a crouching player carry a [`HideSpot`] in
/// their user data.
pub struct StoreKit {
    deps: StoreKitDeps,
    /// shared, so tags can be instanced
    pub tag_geo: Geo,
}

impl StoreKit {
    pub fn new(deps: StoreKitDeps) -> StoreKit {
        StoreKit { deps, tag_geo: plane_geo!(0.2, 0.13) }
    }
    /// The game's kit: `placers::kit::pg`, the given material and tag makers, and `rnd()`.
    pub fn game(mat: Box<dyn FnMut(u32) -> Mat>, tag: Box<dyn FnMut(&TagText) -> Mat>) -> StoreKit {
        StoreKit::new(StoreKitDeps {
            pg: Rc::new(|n: &str, g: Geo| crate::placers::kit::pg(n, g)),
            mat,
            rng: Box::new(crate::base::rng::rnd),
            tag,
        })
    }
    #[inline]
    fn rng(&mut self) -> f64 {
        (self.deps.rng)()
    }
    #[inline]
    fn mat(&mut self, hex: u32) -> Mat {
        (self.deps.mat)(hex)
    }
    #[inline]
    fn pick(&mut self, a: &[u32]) -> u32 {
        let r = self.rng();
        a[(r * a.len() as f64).floor() as usize]
    }
    fn box_(&self, name: &str, w: f64, h: f64, d: f64, m: &Mat, x: f64, y: f64, z: f64) -> Obj {
        let o = mesh((self.deps.pg)(name, box_geo!(w, h, d)), m);
        o.position().set(x, y, z);
        o
    }
    fn cyl(&self, name: &str, rt: f64, rb: f64, h: f64, m: &Mat, x: f64, y: f64, z: f64, seg: u32) -> Obj {
        let o = mesh((self.deps.pg)(name, cyl_geo!(rt, rb, h, seg)), m);
        o.position().set(x, y, z);
        o
    }

    /// Build by name with up to two arguments (`rug(w = 2.2, d = 1.6)`,
    /// `kitchen_run(len = 3.0)`; the rest take no arguments). None for an
    /// unknown name or `price_tag` (which needs its text).
    pub fn build(&mut self, name: &str, a1: Option<f64>, a2: Option<f64>) -> Option<Obj> {
        Some(match name {
            "bed" => self.bed(),
            "nightstand" => self.nightstand(),
            "tableLamp" => self.table_lamp(),
            "wardrobe" => self.wardrobe(),
            "rug" => self.rug(a1.unwrap_or(2.2), a2.unwrap_or(1.6)),
            "sofa" => self.sofa(),
            "armchair" => self.armchair(),
            "coffeeTable" => self.coffee_table(),
            "floorLamp" => self.floor_lamp(),
            "bookcase" => self.bookcase(),
            "tvBench" => self.tv_bench(),
            "kitchenRun" => self.kitchen_run(a1.unwrap_or(3.0)),
            "fridge" => self.fridge(),
            "diningTable" => self.dining_table(),
            "chair" => self.chair(),
            "desk" => self.desk(),
            "officeChair" => self.office_chair(),
            "bunkBed" => self.bunk_bed(),
            "toyBox" => self.toy_box(),
            "rack" => self.rack(),
            "trolley" => self.trolley(),
            _ => return None,
        })
    }

    pub fn bed(&mut self) -> Obj {
        let g = group();
        let wood = {
            let h = self.pick(&WOOD);
            self.mat(h)
        };
        let sheet = {
            let h = self.pick(&[0xe8e4dc, 0xd8dde4, 0xe4d8cc]);
            self.mat(h)
        };
        let cover = {
            let h = self.pick(&FABRIC);
            self.mat(h)
        };
        // a high-legged double bed: there is room underneath (a hiding spot)
        for (x, z) in [(-0.76, -1.0), (0.76, -1.0), (-0.76, 1.0), (0.76, 1.0)] {
            g.add(&self.box_("Store__bed_leg", 0.07, 0.34, 0.07, &wood, x, 0.17, z));
        }
        g.add(&self.box_("Store__bed_frame", 1.64, 0.16, 2.1, &wood, 0.0, 0.42, 0.0));
        g.add(&self.box_("Store__bed_mattress", 1.56, 0.2, 2.0, &sheet, 0.0, 0.6, 0.02));
        g.add(&self.box_("Store__bed_duvet", 1.6, 0.08, 1.35, &cover, 0.0, 0.72, 0.36));
        for x in [-0.4, 0.4] {
            g.add(&self.box_("Store__bed_pillow", 0.62, 0.12, 0.38, &sheet, x, 0.76, -0.72));
        }
        g.add(&self.box_("Store__bed_headboard", 1.7, 1.05, 0.08, &wood, 0.0, 0.72, -1.07));
        set_hide_spot(&g, &HideSpot { eye: 0.26, r: 0.95, kind: "bed".into(), front: None });
        g
    }
    pub fn nightstand(&mut self) -> Obj {
        let g = group();
        let wood = {
            let h = self.pick(&WOOD);
            self.mat(h)
        };
        g.add(&self.box_("Store__stand_body", 0.46, 0.5, 0.4, &wood, 0.0, 0.25, 0.0));
        let m = self.mat(0xf2efe8);
        g.add(&self.box_("Store__stand_drawer", 0.4, 0.14, 0.02, &m, 0.0, 0.36, 0.205));
        g
    }
    pub fn table_lamp(&mut self) -> Obj {
        let g = group();
        let m = self.mat(0x3a3a3a);
        g.add(&self.cyl("Store__lamp_base", 0.07, 0.09, 0.05, &m, 0.0, 0.025, 0.0, 12));
        let m = self.mat(0x3a3a3a);
        g.add(&self.cyl("Store__lamp_stem", 0.012, 0.012, 0.3, &m, 0.0, 0.2, 0.0, 12));
        let m = self.mat(0xf2ead8);
        g.add(&self.cyl("Store__lamp_shade", 0.1, 0.16, 0.2, &m, 0.0, 0.42, 0.0, 16));
        g
    }
    pub fn wardrobe(&mut self) -> Obj {
        let g = group();
        let wood = {
            let h = self.pick(&[0xf0ece4, 0xe0cfb0, 0x5a4636, 0xc9a97a]);
            self.mat(h)
        };
        g.add(&self.box_("Store__wardrobe_body", 1.2, 2.05, 0.6, &wood, 0.0, 1.03, 0.0));
        for x in [-0.3, 0.3] {
            g.add(&self.box_("Store__wardrobe_door", 0.58, 1.95, 0.025, &wood, x, 1.03, 0.312));
        }
        for x in [-0.04, 0.04] {
            let m = self.mat(0x9a9a9a);
            g.add(&self.box_("Store__wardrobe_handle", 0.02, 0.24, 0.03, &m, x, 1.1, 0.34));
        }
        set_hide_spot(&g, &HideSpot { eye: 1.15, r: 0.55, kind: "wardrobe".into(), front: Some(0.35) });
        g
    }
    pub fn rug(&mut self, w: f64, d: f64) -> Obj {
        let g = group();
        let m = {
            let h = self.pick(&[0x8a6a5a, 0x4a5a6a, 0xb8a888, 0x6a7a5a, 0x9a4a3a]);
            self.mat(h)
        };
        g.add(&self.box_("Store__rug", w, 0.012, d, &m, 0.0, 0.006, 0.0));
        g
    }
    pub fn sofa(&mut self) -> Obj {
        let g = group();
        let f = {
            let h = self.pick(&FABRIC);
            self.mat(h)
        };
        let leg = self.mat(0x2a2a2a);
        g.add(&self.box_("Store__sofa_base", 2.1, 0.3, 0.92, &f, 0.0, 0.3, 0.0));
        g.add(&self.box_("Store__sofa_back", 2.1, 0.5, 0.22, &f, 0.0, 0.62, -0.35));
        for x in [-0.97, 0.97] {
            g.add(&self.box_("Store__sofa_arm", 0.18, 0.62, 0.92, &f, x, 0.46, 0.0));
        }
        for x in [-0.58, 0.0, 0.58] {
            g.add(&self.box_("Store__sofa_cushion", 0.58, 0.14, 0.66, &f, x, 0.52, 0.08));
        }
        for (x, z) in [(-0.95, -0.38), (0.95, -0.38), (-0.95, 0.38), (0.95, 0.38)] {
            g.add(&self.box_("Store__sofa_foot", 0.05, 0.15, 0.05, &leg, x, 0.075, z));
        }
        g
    }
    pub fn armchair(&mut self) -> Obj {
        let g = group();
        let f = {
            let h = self.pick(&FABRIC);
            self.mat(h)
        };
        g.add(&self.box_("Store__chair_seat", 0.78, 0.3, 0.82, &f, 0.0, 0.3, 0.0));
        g.add(&self.box_("Store__chair_back", 0.78, 0.55, 0.18, &f, 0.0, 0.62, -0.32));
        for x in [-0.36, 0.36] {
            g.add(&self.box_("Store__chair_arm", 0.12, 0.3, 0.82, &f, x, 0.6, 0.0));
        }
        // short legs: the seat box starts 15 cm up
        for (x, z) in [(-0.34, -0.34), (0.34, -0.34), (-0.34, 0.34), (0.34, 0.34)] {
            let m = self.mat(0x2a2a2a);
            g.add(&self.box_("Store__sofa_foot", 0.05, 0.15, 0.05, &m, x, 0.075, z));
        }
        g
    }
    pub fn coffee_table(&mut self) -> Obj {
        let g = group();
        let w = {
            let h = self.pick(&WOOD);
            self.mat(h)
        };
        g.add(&self.box_("Store__ctable_top", 1.1, 0.05, 0.6, &w, 0.0, 0.43, 0.0));
        g.add(&self.box_("Store__ctable_shelf", 1.0, 0.03, 0.5, &w, 0.0, 0.12, 0.0));
        for (x, z) in [(-0.5, -0.25), (0.5, -0.25), (-0.5, 0.25), (0.5, 0.25)] {
            g.add(&self.box_("Store__ctable_leg", 0.05, 0.43, 0.05, &w, x, 0.215, z));
        }
        g
    }
    pub fn floor_lamp(&mut self) -> Obj {
        let g = group();
        let m = self.mat(0x2e2e2e);
        g.add(&self.cyl("Store__flamp_base", 0.16, 0.18, 0.03, &m, 0.0, 0.015, 0.0, 20));
        g.add(&self.cyl("Store__flamp_pole", 0.015, 0.015, 1.55, &m, 0.0, 0.8, 0.0, 12));
        let s = self.mat(0xefe6d4);
        g.add(&self.cyl("Store__flamp_shade", 0.18, 0.24, 0.3, &s, 0.0, 1.62, 0.0, 20));
        g
    }
    pub fn bookcase(&mut self) -> Obj {
        let g = group();
        let w = {
            let h = self.pick(&WOOD);
            self.mat(h)
        };
        g.add(&self.box_("Store__case_side", 0.03, 1.95, 0.32, &w, -0.4, 0.975, 0.0));
        g.add(&self.box_("Store__case_side", 0.03, 1.95, 0.32, &w, 0.4, 0.975, 0.0));
        g.add(&self.box_("Store__case_back", 0.8, 1.95, 0.01, &w, 0.0, 0.975, -0.155));
        for k in 0..5 {
            g.add(&self.box_("Store__case_shelf", 0.78, 0.025, 0.3, &w, 0.0, 0.05 + k as f64 * 0.46, 0.0));
        }
        // a few display boxes and books
        for k in 0..4 {
            if self.rng() < 0.7 {
                let m = {
                    let h = self.pick(&[0xe8e0d0, 0x9a8a78, 0x4a5a6a, 0xc0b8a8]);
                    self.mat(h)
                };
                let x = if self.rng() < 0.5 { -0.18 } else { 0.18 };
                g.add(&self.box_("Store__case_boxes", 0.3, 0.2, 0.26, &m, x, 0.17 + k as f64 * 0.46, 0.0));
            }
        }
        g
    }
    pub fn tv_bench(&mut self) -> Obj {
        let g = group();
        let w = {
            let h = self.pick(&WOOD);
            self.mat(h)
        };
        g.add(&self.box_("Store__tvbench_body", 1.6, 0.42, 0.4, &w, 0.0, 0.26, 0.0));
        for x in [-0.72, 0.72] {
            let m = self.mat(0x2a2a2a);
            g.add(&self.box_("Store__tvbench_foot", 0.06, 0.05, 0.36, &m, x, 0.025, 0.0));
        }
        let m = self.mat(0x151515);
        g.add(&self.box_("Store__tv_screen", 1.2, 0.7, 0.05, &m, 0.0, 0.85, -0.05));
        let m = self.mat(0x151515);
        g.add(&self.box_("Store__tv_stand", 0.3, 0.02, 0.2, &m, 0.0, 0.48, -0.05));
        g
    }
    pub fn kitchen_run(&mut self, len: f64) -> Obj {
        let g = group();
        let front = {
            let h = self.pick(&[0xf4f2ee, 0x9aa8a0, 0x3a4450, 0xd8c8a8]);
            self.mat(h)
        };
        let top = {
            let h = self.pick(&[0x8a7a66, 0xd8d4cc, 0x2e2e30]);
            self.mat(h)
        };
        let n = engine::math::round_half_up(len / 0.6) as i32;
        for i in 0..n {
            let x = -len / 2.0 + 0.3 + i as f64 * 0.6;
            g.add(&self.box_("Store__kitchen_base", 0.6, 0.86, 0.58, &front, x, 0.43, 0.0));
            g.add(&self.box_("Store__kitchen_wall", 0.6, 0.7, 0.35, &front, x, 1.85, -0.12));
        }
        g.add(&self.box_("Store__kitchen_top", len + 0.02, 0.04, 0.62, &top, 0.0, 0.88, 0.01));
        let m = self.mat(0xb8bcc0);
        g.add(&self.box_("Store__kitchen_sink", 0.5, 0.02, 0.4, &m, -len / 4.0, 0.905, 0.02));
        let m = self.mat(0xb8bcc0);
        g.add(&self.cyl("Store__kitchen_tap", 0.012, 0.015, 0.28, &m, -len / 4.0, 1.04, -0.2, 12));
        let m = self.mat(0x111111);
        g.add(&self.box_("Store__kitchen_hob", 0.58, 0.01, 0.5, &m, len / 4.0, 0.905, 0.02));
        g
    }
    pub fn fridge(&mut self) -> Obj {
        let g = group();
        let m = self.mat(0xe8e8e6);
        g.add(&self.box_("Store__fridge_body", 0.68, 1.85, 0.66, &m, 0.0, 0.925, 0.0));
        let m = self.mat(0x9a9a9a);
        g.add(&self.box_("Store__fridge_handle", 0.03, 0.5, 0.04, &m, 0.28, 1.2, 0.35));
        g
    }
    pub fn dining_table(&mut self) -> Obj {
        let g = group();
        let w = {
            let h = self.pick(&WOOD);
            self.mat(h)
        };
        g.add(&self.box_("Store__dtable_top", 1.8, 0.05, 0.9, &w, 0.0, 0.74, 0.0));
        for (x, z) in [(-0.82, -0.38), (0.82, -0.38), (-0.82, 0.38), (0.82, 0.38)] {
            g.add(&self.box_("Store__dtable_leg", 0.06, 0.72, 0.06, &w, x, 0.36, z));
        }
        g
    }
    pub fn chair(&mut self) -> Obj {
        let g = group();
        let w = {
            let h = self.pick(&WOOD);
            self.mat(h)
        };
        g.add(&self.box_("Store__dchair_seat", 0.44, 0.04, 0.44, &w, 0.0, 0.45, 0.0));
        g.add(&self.box_("Store__dchair_back", 0.44, 0.44, 0.03, &w, 0.0, 0.7, -0.205));
        for (x, z) in [(-0.19, -0.19), (0.19, -0.19), (-0.19, 0.19), (0.19, 0.19)] {
            g.add(&self.box_("Store__dchair_leg", 0.035, 0.45, 0.035, &w, x, 0.225, z));
        }
        g
    }
    pub fn desk(&mut self) -> Obj {
        let g = group();
        let top = {
            let h = self.pick(&[0xf2f0ea, 0xc9a97a, 0x3a3a3a]);
            self.mat(h)
        };
        let leg = self.mat(0x9a9a9a);
        g.add(&self.box_("Store__desk_top", 1.4, 0.03, 0.7, &top, 0.0, 0.74, 0.0));
        for x in [-0.66, 0.66] {
            g.add(&self.box_("Store__desk_leg", 0.05, 0.72, 0.62, &leg, x, 0.36, 0.0));
        }
        g.add(&self.box_("Store__desk_drawers", 0.4, 0.5, 0.6, &top, 0.45, 0.46, 0.0));
        g
    }
    pub fn office_chair(&mut self) -> Obj {
        let g = group();
        let f = {
            let h = self.pick(&[0x2a2a2e, 0x4a5a6a, 0x6a2a2a]);
            self.mat(h)
        };
        let m = self.mat(0x1e1e1e);
        g.add(&self.cyl("Store__ochair_base", 0.3, 0.3, 0.05, &m, 0.0, 0.08, 0.0, 5));
        let m = self.mat(0x5a5a5a);
        g.add(&self.cyl("Store__ochair_post", 0.03, 0.03, 0.34, &m, 0.0, 0.28, 0.0, 12));
        g.add(&self.box_("Store__ochair_seat", 0.5, 0.08, 0.48, &f, 0.0, 0.48, 0.0));
        g.add(&self.box_("Store__ochair_back", 0.46, 0.55, 0.07, &f, 0.0, 0.84, -0.22));
        g
    }
    pub fn bunk_bed(&mut self) -> Obj {
        let g = group();
        let w = {
            let h = self.pick(&[0xf0ece4, 0xc9a97a, 0x8aa0b0]);
            self.mat(h)
        };
        let sheet = {
            let h = self.pick(&[0x6a8ab0, 0xb06a6a, 0xe8e4dc]);
            self.mat(h)
        };
        for (x, z) in [(-0.48, -1.0), (0.48, -1.0), (-0.48, 1.0), (0.48, 1.0)] {
            g.add(&self.box_("Store__bunk_post", 0.06, 1.7, 0.06, &w, x, 0.85, z));
        }
        for y in [0.3, 1.3] {
            g.add(&self.box_("Store__bunk_frame", 1.0, 0.12, 2.06, &w, 0.0, y, 0.0));
            g.add(&self.box_("Store__bunk_mattress", 0.9, 0.14, 1.95, &sheet, 0.0, y + 0.13, 0.0));
        }
        g.add(&self.box_("Store__bunk_rail", 0.03, 0.18, 1.4, &w, 0.5, 1.52, -0.2));
        g
    }
    pub fn toy_box(&mut self) -> Obj {
        let g = group();
        let m = {
            let h = self.pick(&[0xd84a3a, 0x3a7ad8, 0xe8c83a, 0x4ab86a]);
            self.mat(h)
        };
        g.add(&self.box_("Store__toybox", 0.55, 0.36, 0.38, &m, 0.0, 0.18, 0.0));
        g
    }
    /// warehouse pallet racking with cartons
    pub fn rack(&mut self) -> Obj {
        let g = group();
        let steel = self.mat(0x3a5aa8);
        let beam = self.mat(0xd87a2a);
        let carton = self.mat(0xb89a6a);
        for x in [-1.25, 1.25] {
            for z in [-0.45, 0.45] {
                g.add(&self.box_("Store__rack_upright", 0.08, 3.1, 0.08, &steel, x, 1.55, z));
            }
        }
        for y in [0.15, 1.15, 2.15, 3.05] {
            for z in [-0.45, 0.45] {
                g.add(&self.box_("Store__rack_beam", 2.5, 0.1, 0.05, &beam, 0.0, y, z));
            }
        }
        for y in [0.2, 1.2, 2.2] {
            for k in 0..3 {
                if self.rng() < 0.8 {
                    let h = 0.3 + self.rng() * 0.45;
                    g.add(&self.box_("Store__rack_carton", 0.62, h, 0.78, &carton, -0.8 + k as f64 * 0.8, y + 0.05 + h / 2.0, 0.0));
                }
            }
        }
        g
    }
    /// a shopping trolley: wire basket (as a frame), a handle, wheels
    pub fn trolley(&mut self) -> Obj {
        let g = group();
        let wire = self.mat(0x9aa0a6);
        let plastic = self.mat(0x2a5aa8);
        g.add(&self.box_("Store__trolley_basket", 0.55, 0.42, 0.85, &wire, 0.0, 0.72, 0.0));
        g.add(&self.box_("Store__trolley_base", 0.5, 0.03, 0.75, &wire, 0.0, 0.18, 0.0));
        g.add(&self.box_("Store__trolley_handle", 0.56, 0.04, 0.04, &plastic, 0.0, 1.02, -0.47));
        // the chassis: side rails from the wheel base up to the basket, and
        // the handle's two brackets
        for x in [-0.24f64, 0.24] {
            let rail = self.box_("Store__trolley_rail", 0.02, 0.62, 0.02, &wire, x, 0.46, -0.3);
            rail.rotation().x = -0.25;
            g.add(&rail);
            g.add(&self.box_("Store__trolley_rail", 0.02, 0.1, 0.02, &wire, x * 1.08, 0.96, -0.46));
        }
        for (x, z) in [(-0.22, -0.34), (0.22, -0.34), (-0.2, 0.34), (0.2, 0.34)] {
            let m = self.mat(0x1a1a1a);
            let w = self.cyl("Store__trolley_wheel", 0.05, 0.05, 0.03, &m, x, 0.05, z, 12);
            w.rotation().z = PI / 2.0;
            g.add(&w);
        }
        g
    }
    /// a tag on a thin stand beside the thing it prices
    pub fn price_tag(&mut self, text: &TagText) -> Obj {
        let g = group();
        let m = self.mat(0x8a8a8a);
        g.add(&self.cyl("Store__tag_pole", 0.006, 0.006, 0.95, &m, 0.0, 0.475, 0.0, 6));
        let m = self.mat(0x8a8a8a);
        g.add(&self.box_("Store__tag_foot", 0.14, 0.012, 0.14, &m, 0.0, 0.006, 0.0));
        let tm = (self.deps.tag)(text);
        let card = mesh(&self.tag_geo, &tm);
        card.position().set(0, 1.0, 0.01);
        g.add(&card);
        let bm = self.mat(0xe8d84a);
        let back = mesh(&self.tag_geo, &bm);
        back.position().set(0, 1.0, 0.005);
        back.rotation().y = PI;
        g.add(&back);
        g
    }
}

/// One entry of a room-set: the builder's name, x, z, yaw (default 0), the
/// y offset (default 0) and the builder's optional arguments, in a cell's own
/// frame (centre at 0,0, the open side towards +z).
#[derive(Clone, Debug, PartialEq)]
pub struct SetPiece {
    pub name: &'static str,
    pub x: f64,
    pub z: f64,
    pub yaw: f64,
    pub y_off: f64,
    pub a1: Option<f64>,
    pub a2: Option<f64>,
}
fn sp(name: &'static str, x: f64, z: f64, yaw: f64) -> SetPiece {
    SetPiece { name, x, z, yaw, y_off: 0.0, a1: None, a2: None }
}
fn spy(name: &'static str, x: f64, z: f64, yaw: f64, y_off: f64) -> SetPiece {
    SetPiece { name, x, z, yaw, y_off, a1: None, a2: None }
}
fn spa(name: &'static str, x: f64, z: f64, yaw: f64, y_off: f64, a1: f64, a2: Option<f64>) -> SetPiece {
    SetPiece { name, x, z, yaw, y_off, a1: Some(a1), a2 }
}

/// The room-set themes, in their fixed order.
pub const ROOM_SET_NAMES: [&str; 7] = ["bedroom", "living", "kitchen", "dining", "office", "kids", "storage"];
/// The room-set of `theme` (about 6 × 6 m to fill). None for an unknown theme.
pub fn room_set(theme: &str, rng: &mut dyn FnMut() -> f64) -> Option<Vec<SetPiece>> {
    Some(match theme {
        "bedroom" => vec![
            sp("rug", 0.0, 0.3, 0.0),
            sp("bed", 0.0, -0.9, 0.0),
            sp("nightstand", -1.15, -1.75, 0.0),
            sp("nightstand", 1.15, -1.75, 0.0),
            spy("tableLamp", -1.15, -1.75, 0.0, 0.5),
            spy("tableLamp", 1.15, -1.75, 0.0, 0.5),
            sp("wardrobe", 2.3, -1.6, -PI / 2.0),
            if rng() < 0.6 { sp("armchair", -2.1, 0.8, PI / 3.0) } else { sp("floorLamp", -2.2, 0.9, 0.0) },
        ],
        "living" => vec![
            spa("rug", 0.0, 0.2, 0.0, 0.0, 2.6, Some(1.9)),
            sp("sofa", 0.0, -1.3, 0.0),
            sp("coffeeTable", 0.0, 0.1, 0.0),
            sp("armchair", 1.9, 0.2, -PI / 2.0),
            sp("tvBench", 0.0, 1.9, PI),
            sp("floorLamp", -1.55, -1.5, 0.0),
            sp("bookcase", -2.4, 0.2, PI / 2.0),
            if rng() < 0.5 { sp("armchair", -1.8, 1.0, PI / 2.0) } else { sp("floorLamp", 1.6, -1.5, 0.0) },
        ],
        "kitchen" => vec![
            spa("kitchenRun", -0.3, -2.0, 0.0, 0.0, 3.6, None),
            sp("fridge", 1.95, -2.05, 0.0),
            sp("diningTable", 0.0, 0.4, 0.0),
            sp("chair", -0.5, -0.15, 0.0),
            sp("chair", 0.5, -0.15, 0.0),
            sp("chair", -0.5, 0.95, PI),
            sp("chair", 0.5, 0.95, PI),
        ],
        "dining" => vec![
            spa("rug", 0.0, 0.0, 0.0, 0.0, 2.8, Some(2.0)),
            sp("diningTable", 0.0, 0.0, 0.0),
            sp("chair", -0.55, -0.6, 0.0),
            sp("chair", 0.55, -0.6, 0.0),
            sp("chair", -0.55, 0.6, PI),
            sp("chair", 0.55, 0.6, PI),
            sp("chair", -1.15, 0.0, PI / 2.0),
            sp("chair", 1.15, 0.0, -PI / 2.0),
            sp("bookcase", 0.0, -2.2, 0.0),
            sp("bookcase", 0.85, -2.2, 0.0),
        ],
        "office" => vec![
            spa("rug", 0.0, 0.0, 0.0, 0.0, 2.0, Some(1.6)),
            sp("desk", -0.2, -1.2, 0.0),
            sp("officeChair", -0.2, -0.45, PI),
            sp("bookcase", -2.1, -1.3, 0.0),
            sp("bookcase", -1.25, -1.3, 0.0),
            sp("floorLamp", 1.2, -1.5, 0.0),
            sp("armchair", 1.3, 0.8, -PI * 0.75),
        ],
        "kids" => vec![
            spa("rug", 0.0, 0.3, 0.0, 0.0, 2.0, Some(1.6)),
            sp("bunkBed", -1.6, -1.0, 0.0),
            sp("toyBox", 0.6, -1.9, 0.0),
            sp("toyBox", 1.3, -1.9, 0.0),
            sp("wardrobe", 2.3, -0.6, -PI / 2.0),
            sp("chair", 0.7, 0.8, 0.4),
        ],
        "storage" => vec![
            sp("rack", -1.6, -1.6, 0.0),
            sp("rack", 1.4, -1.6, 0.0),
            sp("rack", -1.6, 0.9, 0.0),
            sp("rack", 1.4, 0.9, 0.0),
            sp("trolley", 0.0, 2.1, rng() * 6.0),
        ],
        _ => return None,
    })
}
/// The room-set themes' pick weights, in the order the weighted pick walks them.
pub const ROOM_SET_WEIGHTS: [(&str, f64); 7] =
    [("bedroom", 0.22), ("living", 0.22), ("kitchen", 0.14), ("dining", 0.1), ("office", 0.1), ("kids", 0.08), ("storage", 0.14)];
pub const DEPARTMENT_SIGNS: [(&str, &str); 7] = [
    ("bedroom", "BEDROOMS"),
    ("living", "LIVING ROOMS"),
    ("kitchen", "KITCHENS"),
    ("dining", "DINING"),
    ("office", "WORKSPACES"),
    ("kids", "CHILDREN'S ROOMS"),
    ("storage", "SELF-SERVE"),
];
/// The department sign for `theme`.
pub fn department_sign(theme: &str) -> Option<&'static str> {
    DEPARTMENT_SIGNS.iter().find(|(k, _)| *k == theme).map(|(_, v)| *v)
}
