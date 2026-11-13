//! The props' final look.
//!
//! Placers author flat colours on Lambert surfaces, many of them toy-bright,
//! some with a faint emissive standing in for shine. Once a level's props are
//! placed, `build_world` settles every prop material once: vivid colours are
//! muted toward aged paint and plastic, albedo stays within what real
//! surfaces reflect, the stand-in shine becomes real specular (tinted for
//! brass and steel), plain surfaces get a faint broad sheen, and untextured
//! ones get the shader's wear. Every settled surface, textured or not, takes
//! the shader's wear by use (`Material::wear_use`, `docs/design/PROPS.md` P4): dust
//! on what faces up, the floor's grime at a thing's foot, its edges rubbed
//! and chipped where they are convex. That is in metres over the floor
//! under the thing, so where a thing is decides most of it (a toy on the
//! floor sits in the floor's grime, a frame on a wall has only its top to
//! gather dust); how much it is handled and chipped is judged from its box
//! (`handled`). Those, and its floor, are the thing's own
//! (`Node::use_wear`, set on each of its meshes), never its materials':
//! materials are shared between props, and some across levels
//! (`SharedMats`), so a value kept on one would depend on which prop
//! settled it first and on the levels before. Light sources, see-through
//! and patched materials, pickups and gameplay markers keep their look.
//!
//! Draws no random numbers; materials shared across levels are settled only
//! once (`Material::look_settled`), and what settling does to a material
//! depends on nothing but the material.
use crate::prelude::*;

/// sRGB chroma (largest minus smallest channel) starts to be compressed here
const CHROMA_KNEE: f64 = 0.3;
/// ... and approaches this, never reaching it
const CHROMA_MAX: f64 = 0.52;
/// albedo (linear, largest channel) rolls off from here toward ALBEDO_MAX
const ALBEDO_KNEE: f64 = 0.58;
const ALBEDO_MAX: f64 = 0.8;
/// the darkest albedo, approached softly from above
const ALBEDO_MIN: f64 = 0.02;
/// an emissive this faint (linear, largest channel) on a lit prop is a stand-in for shine
const FAKE_SHINE_MAX: f64 = 0.035;

/// What a surface's highlight looks like
#[derive(Clone, Copy, Debug, PartialEq)]
enum Finish {
    /// paint, wood, paper, cloth, matte plastic: a faint broad sheen
    Plain,
    /// glazed ceramic, glossy plastic, varnish
    Gloss,
    /// steel, chrome, aluminium: tinted highlight, darker diffuse
    Steel,
    /// brass, gold, copper
    Brass,
}

fn srgb_of(c: &Color) -> [f64; 3] {
    [linear_to_srgb(c.r.max(0.0)), linear_to_srgb(c.g.max(0.0)), linear_to_srgb(c.b.max(0.0))]
}

/// The colour muted and kept within real-world albedo.
pub fn settle_albedo(c: &Color) -> Color {
    let mut s = srgb_of(c);
    let (hi, lo) = (s[0].max(s[1]).max(s[2]), s[0].min(s[1]).min(s[2]));
    let chroma = hi - lo;
    if chroma > CHROMA_KNEE {
        // pull toward the colour's own grey: hue and lightness stay, chroma rolls off
        let span = CHROMA_MAX - CHROMA_KNEE;
        let want = CHROMA_KNEE + span * (1.0 - (-(chroma - CHROMA_KNEE) / span).exp());
        let luma = 0.2126 * s[0] + 0.7152 * s[1] + 0.0722 * s[2];
        let k = want / chroma;
        for v in s.iter_mut() {
            *v = luma + (*v - luma) * k;
        }
    }
    let mut out = Color::rgb_linear(srgb_to_linear(s[0]), srgb_to_linear(s[1]), srgb_to_linear(s[2]));
    let m = out.r.max(out.g).max(out.b);
    let want = if m > ALBEDO_KNEE {
        let span = ALBEDO_MAX - ALBEDO_KNEE;
        ALBEDO_KNEE + span * (1.0 - (-(m - ALBEDO_KNEE) / span).exp())
    } else {
        // a soft floor: (m⁴ + min⁴)^¼ is m well above the floor and the floor at 0
        (m.powi(4) + ALBEDO_MIN.powi(4)).powf(0.25)
    };
    if m > 1e-6 {
        let k = want / m;
        out.set_rgb(out.r * k, out.g * k, out.b * k);
    } else {
        out.set_scalar(want);
    }
    out
}

/// The finish a stand-in shine meant, judged by the surface colour.
fn finish_of_shine(c: &Color) -> Finish {
    let s = srgb_of(c);
    let (hi, lo) = (s[0].max(s[1]).max(s[2]), s[0].min(s[1]).min(s[2]));
    let chroma = hi - lo;
    if chroma < 0.12 {
        if hi > 0.45 {
            Finish::Steel
        } else {
            Finish::Gloss
        }
    } else if s[0] >= s[1] && s[1] >= s[2] && chroma >= 0.2 && (0.35..0.85).contains(&hi) {
        // warm metal: red ≥ green ≥ blue, not a pale cream
        Finish::Brass
    } else {
        Finish::Gloss
    }
}

/// Whether an unlit colour is meant to glow (or to be a void): voids,
/// near-white light, and bright saturated indicator colours stay unlit.
fn keeps_unlit(c: &Color) -> bool {
    let s = srgb_of(c);
    let (hi, lo) = (s[0].max(s[1]).max(s[2]), s[0].min(s[1]).min(s[2]));
    !(0.12..0.96).contains(&hi) || (hi >= 0.8 && hi - lo >= 0.5)
}

/// The highlight of a finish: (specular, shininess, diffuse scale)
fn highlight(finish: Finish, c: &Color) -> (Color, f64, f64) {
    match finish {
        Finish::Plain => (Color::rgb_linear(0.035, 0.035, 0.035), 14.0, 1.0),
        Finish::Gloss => (Color::rgb_linear(0.05, 0.05, 0.05), 55.0, 1.0),
        Finish::Steel => (Color::rgb_linear(c.r * 0.6, c.g * 0.6, c.b * 0.6), 48.0, 0.7),
        Finish::Brass => (Color::rgb_linear(c.r * 0.55, c.g * 0.55, c.b * 0.55), 40.0, 0.78),
    }
}

/// How much a prop is handled and how much its paint is chipped, from its
/// box (its lift off the floor under it, its height and its largest side,
/// metres): furniture standing on the floor is knocked, leaned on and
/// dragged (1, 1); a small thing on the floor is scuffed rather than
/// chipped, as moulded and stuffed things are (0.7, 0.25); a thing hung up
/// out of reach (a frame, a clock, a shelf's things) is only dusted (0.45,
/// no chips)
fn handled(lift: f64, height: f64, size: f64) -> (f64, f64) {
    if lift > 0.3 {
        (0.45, 0.0)
    } else if height < 0.45 && size < 0.6 {
        (0.7, 0.25)
    } else {
        (1.0, 1.0)
    }
}

/// Settle one material; returns false when it is left alone. What it does
/// depends on the material alone (not on the prop: a material is shared).
fn settle_material(m: &Mat) -> bool {
    let mut mm = m.m();
    if mm.look_settled {
        return false;
    }
    mm.look_settled = true;
    if !matches!(mm.kind, MatKind::Basic | MatKind::Lambert | MatKind::Standard | MatKind::Phong)
        || mm.patch != Patch::None
        || mm.transparent
        || mm.blending != Blending::Normal
        || mm.vertex_colors
        || mm.emissive_map.is_some()
        || !mm.fog
        || !mm.depth_test
        || !mm.color_write
    {
        return false;
    }
    let textured = mm.map.is_some();
    let finish = if mm.kind == MatKind::Basic {
        // a painted part drawn unlit (label, gauge, a tube lying on the floor)
        // reads as a sticker: light it like the rest of the prop
        if textured || keeps_unlit(&mm.color) {
            return false;
        }
        Finish::Gloss
    } else {
        let e = mm.emissive;
        let shine = e.r.max(e.g).max(e.b) * mm.emissive_intensity;
        if shine > FAKE_SHINE_MAX {
            // a light source (lamp shade, lit sign): keep the glow, mute the colour
            if !textured {
                mm.color = settle_albedo(&mm.color);
            }
            return true;
        }
        mm.emissive = Color::default();
        if shine > 0.0 {
            finish_of_shine(&mm.color)
        } else {
            Finish::Plain
        }
    };
    // textured surfaces keep their own colours (the shader only reins in the
    // most vivid ones) and get lighter wear
    if textured {
        mm.wear = 0.5;
    } else {
        mm.color = settle_albedo(&mm.color);
        mm.wear = 1.0;
    }
    // wear by use, whole on a texture too: what was painted on it is its
    // own, the dust and the floor's grime come after (how much is the
    // thing's: `Node::use_wear`)
    mm.wear_use = 1.0;
    if mm.kind == MatKind::Phong {
        return true; // keeps its own highlight
    }
    let c = mm.color;
    let (spec, shininess, diffuse_k) = highlight(finish, &c);
    mm.kind = MatKind::Phong;
    mm.specular = spec;
    mm.shininess = shininess;
    mm.color.set_rgb(c.r * diffuse_k, c.g * diffuse_k, c.b * diffuse_k);
    true
}

/// A placed prop measured once, for its look and its contact shadow
/// (`visuals::contact`): its world box and the floor under its middle (0
/// when the box is empty)
pub struct PropBox {
    pub top: Obj,
    pub bb: Box3,
    pub floor: f64,
}

impl Game {
    /// The props from the world's child `start` on that are settled
    /// (pickups and gameplay markers keep their look), each measured where
    /// it now lies
    pub fn prop_boxes(&self, start: usize) -> Vec<PropBox> {
        let keep: Vec<Obj> = self.almonds.iter().chain(self.battery_items.iter()).map(|p| p.g.clone()).collect();
        let mut out = vec![];
        for top in self.world.children().into_iter().skip(start) {
            if top.user().gameplay || keep.iter().any(|k| k.ptr_eq(&top)) {
                continue;
            }
            out.push(self.prop_box(top));
        }
        out
    }

    /// `top` measured: its world box and the floor under its middle
    fn prop_box(&self, top: Obj) -> PropBox {
        let mut bb = Box3::empty();
        bb.set_from_object(&top);
        let floor = if bb.is_empty() { 0.0 } else { self.floor_y_at((bb.min.x + bb.max.x) / 2.0, (bb.min.z + bb.max.z) / 2.0) };
        PropBox { top, bb, floor }
    }

    /// Settle the look of every prop material under `props` (`prop_boxes`)
    /// and set each prop's wear by use on its meshes (see the module notes).
    /// Returns the number of materials changed.
    pub fn settle_prop_looks(&mut self, props: &[PropBox]) -> usize {
        props.iter().map(|p| self.settle_look_with(p)).sum()
    }

    /// Settle the look of one prop (`top` and everything under it, placed
    /// in the world) and set its wear by use; returns the number of
    /// materials changed. The furniture the Building's rooms are dressed
    /// with is settled so, as it is made (`building_dress::dress_rooms`)
    pub fn settle_look_of(&self, top: &Obj) -> usize {
        self.settle_look_with(&self.prop_box(top.clone()))
    }

    fn settle_look_with(&self, p: &PropBox) -> usize {
        let mut n = 0;
        let wear = if p.bb.is_empty() {
            UseWear::default()
        } else {
            let size = p.bb.get_size();
            let (handled, chips) = handled(p.bb.min.y - p.floor, size.y, size.x.max(size.y).max(size.z));
            UseWear { handled, chips, floor: p.floor }
        };
        p.top.traverse(&mut |o: &Obj| {
            if !o.is_mesh() {
                return;
            }
            o.set_use_wear(wear);
            for m in o.mats() {
                if settle_material(&m) {
                    n += 1;
                }
            }
        });
        n
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn chroma(c: &Color) -> f64 {
        let s = srgb_of(c);
        s[0].max(s[1]).max(s[2]) - s[0].min(s[1]).min(s[2])
    }
    #[test]
    fn vivid_colours_are_muted_and_neutrals_kept() {
        for hex in [0xc02020u32, 0x2060c0, 0x40a040, 0xf0e020, 0xd050a0, 0xf0d020, 0xff0000] {
            let c = settle_albedo(&Color::hex(hex));
            assert!(chroma(&c) < CHROMA_MAX, "{hex:06x}: chroma {}", chroma(&c));
            assert!(chroma(&c) > 0.3, "{hex:06x}: still coloured");
        }
        for hex in [0x4c3b22u32, 0x3a3f45, 0x9a774a, 0x808080] {
            let (a, b) = (Color::hex(hex), settle_albedo(&Color::hex(hex)));
            assert!((a.r - b.r).abs() + (a.g - b.g).abs() + (a.b - b.b).abs() < 1e-3, "{hex:06x} changed");
        }
    }
    #[test]
    fn albedo_stays_in_range() {
        let w = settle_albedo(&Color::hex(0xffffff));
        assert!(w.r < ALBEDO_MAX && w.r > 0.7);
        let k = settle_albedo(&Color::hex(0x000000));
        assert!((k.r - ALBEDO_MIN).abs() < 1e-9);
        let d = settle_albedo(&Color::hex(0x0b0b0e));
        assert!(d.b >= ALBEDO_MIN && d.b < 0.03);
    }
    #[test]
    fn stand_in_shine_becomes_specular_once() {
        let brass = lambert!({ color: 0xa07030, emissive: 0x1a1005 });
        assert!(settle_material(&brass));
        let (kind, e, spec) = (brass.r().kind, brass.emissive_v(), brass.r().specular);
        assert_eq!(kind, MatKind::Phong);
        assert_eq!(e.get_hex(), 0);
        assert!(spec.r > spec.b * 2.0, "brass highlight is warm");
        let c = brass.color_v();
        assert!(!settle_material(&brass), "settled twice");
        assert_eq!(brass.color_v().get_hex(), c.get_hex());
        // a lamp keeps its glow
        let lamp = lambert!({ color: 0xefe2c4, emissive: 0xffc070, emissiveIntensity: 0.9 });
        settle_material(&lamp);
        assert_eq!(lamp.r().kind, MatKind::Lambert);
        assert_eq!(lamp.emissive_v().get_hex(), 0xffc070);
        // indicator lights, voids and see-through materials are left alone
        for hex in [0xff2020u32, 0x40ff40, 0xfff2c8, 0x080604] {
            let led = basic_mat!({ color: hex });
            assert!(!settle_material(&led));
            assert_eq!((led.r().kind, led.color_v().get_hex()), (MatKind::Basic, hex));
        }
        // a painted label drawn unlit gets lit
        let label = basic_mat!({ color: 0xf0ece0 });
        assert!(settle_material(&label));
        assert_eq!(label.r().kind, MatKind::Phong);
        let glass = lambert!({ color: 0x2a3438, emissive: 0x0a0e10, transparent: true, opacity: 0.55 });
        assert!(!settle_material(&glass));
        assert_eq!(glass.emissive_v().get_hex(), 0x0a0e10);
    }
    #[test]
    fn wear_by_use_follows_what_the_thing_is() {
        // a filing cabinet, a toy on the floor, a frame on a wall: handled,
        // and chipped (the toy scuffed rather than chipped, the frame not)
        assert_eq!(handled(0.0, 1.3, 1.3), (1.0, 1.0));
        assert_eq!(handled(0.0, 0.14, 0.14), (0.7, 0.25));
        assert_eq!(handled(1.4, 0.3, 0.55), (0.45, 0.0));
        // a printed surface keeps its own colours and light mottle, but
        // takes dust and grime by use like the rest; a lamp does not
        let tex = Tex::data_rgba(vec![128; 4 * 4 * 4], 4, 4);
        let print = lambert!({ map: &tex });
        assert!(settle_material(&print));
        assert_eq!((print.r().wear, print.r().wear_use), (0.5, 1.0));
        let lamp = lambert!({ color: 0xefe2c4, emissive: 0xffc070 });
        settle_material(&lamp);
        assert_eq!(lamp.r().wear_use, 0.0);
    }
}
