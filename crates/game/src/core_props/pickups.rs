//! Almond water and the battery: the two things every player picks up, made
//! by the asset pipeline (`docs/design/ASSETS.md`; their dossiers,
//! `assets/src/props/almond_water/` and `assets/src/props/battery/`, hold
//! every measurement's source). One builder each, for every place that makes
//! one (the floor's own, a drawer's, a camp's, a machine's spill, a door's
//! gift, the Building's, the Loop's instances, the yellowish one put
//! away): the real thing at its real size, still where it
//! lies (nothing floats or turns), in what it is made of (clear PET, its
//! water faintly milky; a white HDPE cap; a D cell's printed film and nickel
//! ends), nothing printed on the bottle (the canon: `VENDING.md`).
//!
//! Every bottle is sealed and full (`LORE.md`, *Survival*): the one almond
//! water opened is the yellowish one put away (`secrets::things`). What
//! else one is like (standing or on its side, an old cell's leak, its turn)
//! is decided by where it lies, on a stream of its own (`SplitMix64`, never
//! the level's: no floor's draws move).
use crate::assets::props::prop_part_as_modelled;
use crate::base::rng::SplitMix64;
use crate::placers::kit::canvas_tex_opt;
use crate::prelude::*;

/// A bottle's height to the top of its neck, and its cap's (`AlmondWater__*`)
pub const BOTTLE_H: f64 = 0.203;
pub const BOTTLE_R: f64 = 0.0325;
const CAP_H: f64 = 0.0122;
const BAND_H: f64 = 0.0032;
/// A D cell's radius and length (`Battery__*`): 33.2 x 61 mm
pub const CELL_R: f64 = 0.0166;
pub const CELL_L: f64 = 0.061;
const JACKET_L: f64 = 0.0585;

/// Whether a bottle was opened
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fill {
    /// sealed, its cap on and its band with it: every almond water
    /// (`LORE.md`, *Survival*; what is said as one is picked up)
    Sealed,
    /// its seal broken, the cap screwed back a little off true, its band left
    /// on the neck: only the yellowish one put away (*What is put away*: its
    /// tell)
    Opened,
}

/// What a bottle or a cell lying at (x, z) is like: whether it lies on its
/// side, an old cell's leak, and its turn; from its place alone (every bit
/// of both, to the centimetre: things on the Building's 4 m grid differ)
pub fn state_at(x: f64, z: f64, salt: u64) -> (bool, bool, f64) {
    let (xi, zi) = ((x * 100.0).round() as i64 as u64, (z * 100.0).round() as i64 as u64);
    let h = xi.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ zi.wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    let mut r = SplitMix64::new((h ^ (h >> 32)) as u32, salt);
    let lying = r.next() < 0.25;
    let leaked = r.next() < 0.3;
    let turn = r.next() * std::f64::consts::TAU;
    (lying, leaked, turn)
}
const BOTTLE_SALT: u64 = 0xA1_0D_5A_7E_12_0B_07_71;
const CELL_SALT: u64 = 0xBA_77_E2_1D_C3_11_0D_05;

/// A part as modelled, or the box filling its frame when the parts are off
fn part(name: &str) -> Geo {
    prop_part_as_modelled(name).unwrap_or_else(|| panic!("no prop part {name} (cargo xtask assets props)"))
}

/// The water, its own copy moved so its origin is its bottle's: the two tie
/// on depth, and the water, made first, is drawn first (inside it), while
/// both sort by depth with everything else (a render order would draw every
/// bottle's water before every other see-through thing)
fn water_geo() -> Geo {
    let make = || {
        let g = part("AlmondWater__liquidFull").duplicate();
        g.translate(0.0, WATER_TOP / 2.0 + 0.003 - BOTTLE_H / 2.0, 0.0);
        g
    };
    // one copy a thread, shared as the parts are (`prop_part_as_modelled`'s
    // cache); with the parts off, a box made each time as theirs are
    if !crate::assets::props::props_enabled() {
        return make();
    }
    thread_local! {
        static WATER: std::cell::OnceCell<Geo> = const { std::cell::OnceCell::new() };
    }
    WATER.with(|c| c.get_or_init(make).clone())
}
/// The water's top, in a full bottle
const WATER_TOP: f64 = 0.165;
/// The yellowish water of the one put away
fn yellow() -> Mat {
    phong!({ color: 0xd6c27a, transparent: true, opacity: 0.28, depthWrite: false, specular: 0x303030, shininess: 60 })
}

fn pet() -> Mat {
    // clear PET with a faint blue toner: the light off its ribs, little else
    phong!({ color: 0xf4f8fc, transparent: true, opacity: 0.12, depthWrite: false, specular: 0x6a6e72, shininess: 90 })
}
fn water() -> Mat {
    // water with a little milk in it: very pale, faintly cloudy
    phong!({ color: 0xe9e9d7, transparent: true, opacity: 0.16, depthWrite: false, specular: 0x303030, shininess: 60 })
}
fn hdpe() -> Mat {
    // a white HDPE cap: waxy, half matte
    phong!({ color: 0xdedcd6, specular: 0x202020, shininess: 14 })
}
fn nickel() -> Mat {
    phong!({ color: 0xb8b2a8, specular: 0x9a9690, shininess: 60 })
}
fn crust() -> Mat {
    lambert!({ color: 0xd8d6cc })
}

thread_local! {
    static LABEL: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() };
}

/// A D cell's printed film, as its jacket's texture wraps it (u round it, v
/// from the negative end up): a nobody's make, HALDEN (as the canon's makers
/// are nobody's: `VENDING.md`), its metallised ground and a band of colour
/// at the positive end, the size letter on the band, its type along it, the
/// caution panel round the back, a lot code by the negative end (no date:
/// the place keeps none), scuffed at the shoulders
fn label_tex() -> Tex {
    LABEL.with(|c| {
        c.get_or_init(|| {
            canvas_tex_opt(512, 256, false, |ctx, w, h| {
                // the metallised ground, brushed
                let g = ctx.create_linear_gradient(0, 0, 0, h);
                g.add_color_stop(0.0, "#9a9a96");
                g.add_color_stop(0.5, "#b4b4ae");
                g.add_color_stop(1.0, "#8e8e8a");
                ctx.set_fill_style(&g);
                ctx.fill_rect(0, 0, w, h);
                for k in 0..140 {
                    let y = rnd() * h;
                    ctx.set_fill_style(if k % 2 == 0 { "rgba(255,255,255,0.05)" } else { "rgba(0,0,0,0.05)" });
                    ctx.fill_rect(0, y, w, 1.0);
                }
                // the band at the positive end (the top of the texture is v = 1)
                let band = h * 0.34;
                ctx.set_fill_style("#2f4a3c");
                ctx.fill_rect(0, 0, w, band);
                ctx.set_fill_style("rgba(210,200,160,0.85)");
                ctx.fill_rect(0, band - 3.0, w, 3.0);
                // the size letter, boxed, twice round
                for x0 in [w * 0.12, w * 0.62] {
                    ctx.set_stroke_style("#e8e4d6");
                    ctx.set_line_width(3.0);
                    ctx.stroke_rect(x0, band * 0.16, band * 0.68, band * 0.68);
                    ctx.set_fill_style("#e8e4d6");
                    ctx.set_font("bold 64px Helvetica");
                    ctx.fill_text("D", x0 + band * 0.14, band * 0.74);
                    ctx.set_font("bold 30px Helvetica");
                    ctx.fill_text("+", x0 + band * 0.9, band * 0.62);
                }
                // the make and its type, along it
                ctx.set_fill_style("#1f2a24");
                ctx.set_font("bold 34px Helvetica");
                ctx.fill_text("HALDEN", w * 0.06, h * 0.55);
                ctx.set_font("bold 22px Helvetica");
                ctx.fill_text("ALKALINE", w * 0.06, h * 0.68);
                ctx.set_font("18px Helvetica");
                ctx.fill_text("LR20  1.5V", w * 0.06, h * 0.78);
                // the caution panel round the back
                ctx.set_font("9px Helvetica");
                for (k, line) in [
                    "CAUTION: DO NOT RECHARGE.",
                    "DO NOT DISPOSE OF IN FIRE.",
                    "INSERT CORRECTLY: + AND -.",
                    "REMOVE FROM A DEVICE STORED UNUSED.",
                ]
                .iter()
                .enumerate()
                {
                    ctx.fill_text(line, w * 0.55, h * 0.52 + k as f64 * 12.0);
                }
                ctx.set_font("bold 14px Helvetica");
                ctx.fill_text("-", w * 0.3, h * 0.93);
                ctx.set_font("10px Courier");
                ctx.fill_text("LOT 7G38", w * 0.78, h * 0.94);
                // scuffs at both shoulders and along it
                for k in 0..90 {
                    let y = if k % 3 == 0 {
                        rnd() * h
                    } else if k % 3 == 1 {
                        rnd() * h * 0.08
                    } else {
                        h - rnd() * h * 0.08
                    };
                    ctx.set_fill_style("rgba(225,222,214,0.35)");
                    ctx.fill_rect(rnd() * w, y, 2.0 + rnd() * 9.0, 1.0);
                }
            })
        })
        .clone()
    })
}

fn add(g: &Obj, geo: Geo, mat: &Mat, y: f64) -> Obj {
    let m = mesh(&geo, mat);
    m.position().y = y;
    g.add(&m);
    m
}

/// A bottle (`Game::almond_water`) laid on its side, or stood up
pub fn lay(bottle: &Obj, lying: bool) {
    let Some(inner) = bottle.children().first().cloned() else { return };
    if lying {
        inner.rotation().z = -std::f64::consts::FRAC_PI_2;
        inner.position().set(-BOTTLE_H / 2.0, BOTTLE_R, 0.0);
    } else {
        inner.rotation().z = 0.0;
        inner.position().set(0.0, 0.0, 0.0);
    }
}

impl Game {
    /// A bottle of almond water in its `fill` state, its base at the group's
    /// origin, standing (or, `lying`, on its side along x, resting on the
    /// floor)
    pub fn almond_water(&self, fill: Fill, lying: bool) -> Obj {
        let g = group();
        let inner = group();
        let mid = BOTTLE_H / 2.0;
        // full, whether sealed or not: the yellowish one put away was never
        // drunk from, only opened (its tell); made before its bottle, drawn
        // inside it (`water_geo`)
        add(&inner, water_geo(), &if fill == Fill::Opened { yellow() } else { water() }, mid);
        add(&inner, part("AlmondWater__bottle"), &pet(), mid);
        let cap = add(&inner, part("AlmondWater__cap"), &hdpe(), BOTTLE_H + CAP_H / 2.0 - 0.0094);
        if fill == Fill::Opened {
            // screwed back a little off true
            cap.rotation().z = 0.07;
            cap.position().x = 0.0012;
        }
        // the band: under the cap on a sealed bottle, on the neck's support
        // ring of an opened one
        let band_y = if fill == Fill::Sealed { BOTTLE_H - 0.0094 - BAND_H / 2.0 } else { 0.1880 + BAND_H / 2.0 };
        add(&inner, part("AlmondWater__tamperRing"), &hdpe(), band_y);
        g.add(&inner);
        // on its side: its axis along x, resting on its body
        lay(&g, lying);
        g.user().s.insert("kind".into(), "almond_water".into());
        g
    }

    /// A D cell lying on its side along x on what it rests on (the group's
    /// origin), `leaked` with the crust an old one grows at its negative end
    pub fn battery(&self, leaked: bool) -> Obj {
        let g = group();
        let cell = group();
        add(&cell, part("Battery__jacket"), &phong!({ map: label_tex(), specular: 0x505050, shininess: 70 }), JACKET_L / 2.0);
        add(&cell, part("Battery__negative"), &nickel(), 0.0);
        add(&cell, part("Battery__positive"), &nickel(), JACKET_L + 0.0011);
        if leaked {
            add(&cell, part("Battery__crystals"), &crust(), 0.0012);
        }
        // its axis along x, resting on the jacket
        cell.rotation().z = -std::f64::consts::FRAC_PI_2;
        cell.position().set(-CELL_L / 2.0, CELL_R, 0.0);
        g.add(&cell);
        g.user().s.insert("kind".into(), "battery".into());
        g
    }

    /// The bottle a place gives at (x, z): its state from where it is
    pub fn almond_water_at(&self, x: f64, z: f64) -> Obj {
        let (lying, _, turn) = state_at(x, z, BOTTLE_SALT);
        let g = self.almond_water(Fill::Sealed, lying);
        g.rotation().y = turn;
        g
    }

    /// The cell a place gives at (x, z); `fresh`, never an old one's leak
    /// (a machine's stock)
    pub fn battery_at(&self, x: f64, z: f64, fresh: bool) -> Obj {
        let (_, leaked, turn) = state_at(x, z, CELL_SALT);
        let g = self.battery(leaked && !fresh);
        g.rotation().y = turn;
        g
    }
}

/// Geometries already in place, as one
fn merge(gs: &[Geo]) -> Geo {
    let (mut pos, mut nrm, mut uv, mut idx) = (vec![], vec![], vec![], vec![]);
    for geo in gs {
        let g = geo.r();
        let base = (pos.len() / 3) as u32;
        let n = g.position.len() / 3;
        pos.extend_from_slice(&g.position);
        nrm.extend_from_slice(&g.normal);
        if g.uv.len() >= n * 2 {
            uv.extend_from_slice(&g.uv[..n * 2]);
        } else {
            uv.extend(std::iter::repeat_n(0.0, n * 2));
        }
        match &g.index {
            Some(ix) => idx.extend(ix.iter().map(|&i| i + base)),
            None => idx.extend((0..n as u32).map(|i| i + base)),
        }
    }
    let out = Geo::new();
    out.set_position(pos).set_normal(nrm).set_uv(uv).set_index(idx);
    out.compute_bounding_box();
    out.compute_bounding_sphere();
    out
}

/// A sealed bottle standing and a fresh cell lying as the builders make
/// them, each material's parts in one geometry of their own (copies, the
/// caller's to dispose) in the thing's frame, in drawing order: for a place
/// that draws many at once (the Loop's instances)
pub fn instanced_parts() -> (Vec<(Geo, Mat)>, Vec<(Geo, Mat)>) {
    let at = |name: &str, y: f64| {
        let g = part(name).duplicate();
        g.translate(0.0, y, 0.0);
        g
    };
    let mid = BOTTLE_H / 2.0;
    let white =
        merge(&[at("AlmondWater__cap", BOTTLE_H + CAP_H / 2.0 - 0.0094), at("AlmondWater__tamperRing", BOTTLE_H - 0.0094 - BAND_H / 2.0)]);
    let liquid = water_geo().duplicate();
    liquid.translate(0.0, mid, 0.0);
    let bottle = vec![(liquid, water()), (at("AlmondWater__bottle", mid), pet()), (white, hdpe())];
    // the cell along x on its jacket, as `Game::battery` lays it
    let lay = |g: Geo| {
        g.rotate_z(-std::f64::consts::FRAC_PI_2);
        g.translate(-CELL_L / 2.0, CELL_R, 0.0);
        g
    };
    let jacket = lay(at("Battery__jacket", JACKET_L / 2.0));
    let ends = lay(merge(&[at("Battery__negative", 0.0), at("Battery__positive", JACKET_L + 0.0011)]));
    let cell = vec![(jacket, phong!({ map: label_tex(), specular: 0x505050, shininess: 70 })), (ends, nickel())];
    (bottle, cell)
}
